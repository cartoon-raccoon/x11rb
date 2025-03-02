// This code is dual licensed under MIT OR Apache 2.0.

//! Cookies!

use x11rb::connection::{BufWithFds, ReplyOrError, RequestKind};
use x11rb_protocol::protocol::xproto::ListFontsWithInfoReply;
use x11rb_protocol::{DiscardMode, SequenceNumber};

use crate::connection::{Connection, RequestConnection};
use crate::errors::{ConnectionError, ReplyError};
use crate::x11_utils::{TryParse, TryParseFd};

use futures_lite::{ready, stream::Stream};
use std::future::Future;
use std::marker::PhantomData;
use std::mem;
use std::pin::Pin;
use std::task::{Context, Poll};

#[cfg(feature = "record")]
use crate::protocol::record::EnableContextReply;

/// A cookie for a request without a reply.
#[derive(Debug)]
pub struct VoidCookie<'conn, C: RequestConnection + ?Sized> {
    conn: &'conn C,
    sequence: SequenceNumber,
}

impl<'conn, C: Connection + ?Sized> VoidCookie<'conn, C> {
    /// Create a new cookie from its raw parts.
    pub fn new(conn: &'conn C, sequence: SequenceNumber) -> Self {
        Self { conn, sequence }
    }

    /// Get the sequence number of this cookie.
    pub fn sequence_number(&self) -> SequenceNumber {
        self.sequence
    }

    /// Check if this request caused an X11 error.
    pub async fn check(self) -> Result<(), ReplyError> {
        let res = self.conn.check_for_raw_error(self.sequence).await;

        // Wait until after the `await` to consume the cookie.
        let (conn, _) = self.consume();

        match res? {
            Some(e) => Err(conn.parse_error(e.as_ref())?.into()),
            None => Ok(()),
        }
    }

    /// Ignore errors associated with this request.
    pub fn ignore_error(self) {
        let (conn, seq) = self.consume();
        conn.discard_reply(seq, RequestKind::IsVoid, DiscardMode::DiscardReplyAndError);
    }

    /// Eat the cookie and return the connection.
    fn consume(self) -> (&'conn C, SequenceNumber) {
        let res = (self.conn, self.sequence);
        mem::forget(self);
        res
    }
}

impl<C: RequestConnection + ?Sized> Drop for VoidCookie<'_, C> {
    fn drop(&mut self) {
        self.conn.discard_reply(
            self.sequence,
            RequestKind::IsVoid,
            DiscardMode::DiscardReply,
        );
    }
}

/// Helper for cookies that hold a reply.
#[derive(Debug)]
struct RawCookie<'a, C: RequestConnection + ?Sized> {
    conn: &'a C,
    sequence: SequenceNumber,
}

impl<'a, C: RequestConnection + ?Sized> RawCookie<'a, C> {
    fn new(conn: &'a C, sequence: SequenceNumber) -> Self {
        Self { conn, sequence }
    }

    fn consume(self) -> (&'a C, SequenceNumber) {
        let res = (self.conn, self.sequence);
        mem::forget(self);
        res
    }
}

impl<C: RequestConnection + ?Sized> Drop for RawCookie<'_, C> {
    fn drop(&mut self) {
        self.conn.discard_reply(
            self.sequence,
            RequestKind::HasResponse,
            DiscardMode::DiscardReply,
        );
    }
}

/// A cookie for a request that has a reply.
#[derive(Debug)]
pub struct Cookie<'conn, C: RequestConnection + ?Sized, R> {
    raw: RawCookie<'conn, C>,
    capture: PhantomData<R>,
}

impl<'conn, C: Connection + ?Sized, R: TryParse> Cookie<'conn, C, R> {
    /// Create a new cookie from its raw parts.
    pub fn new(conn: &'conn C, sequence: SequenceNumber) -> Self {
        Self {
            raw: RawCookie::new(conn, sequence),
            capture: PhantomData,
        }
    }

    /// Get the sequence number of this cookie.
    pub fn sequence_number(&self) -> SequenceNumber {
        self.raw.sequence
    }

    /// Get the raw reply that the server sent.
    pub async fn raw_reply(self) -> Result<C::Buf, ReplyError> {
        // Wait for the reply
        let reply_or_error = self
            .raw
            .conn
            .wait_for_reply_or_raw_error(self.raw.sequence)
            .await;

        // Wait until after the `await` to consume the cookie.
        let (conn, _) = self.raw.consume();

        // Check for errors
        match reply_or_error? {
            ReplyOrError::Reply(reply) => Ok(reply),
            ReplyOrError::Error(error) => Err(conn.parse_error(error.as_ref())?.into()),
        }
    }

    /// Get the reply, but have errors handled as events.
    pub async fn raw_reply_unchecked(self) -> Result<Option<C::Buf>, ConnectionError> {
        // Wait for the reply
        let res = self.raw.conn.wait_for_reply(self.raw.sequence).await;

        // Wait until after the `await` to consume the cookie.
        let _ = self.raw.consume();

        // Check for errors
        res
    }

    /// Get the reply that the server sent.
    pub async fn reply(self) -> Result<R, ReplyError> {
        let buf = self.raw_reply().await?;

        // Parse the reply
        let (reply, _) = R::try_parse(buf.as_ref())?;
        Ok(reply)
    }

    /// Get the reply, but have errors handled as events.
    pub async fn reply_unchecked(self) -> Result<Option<R>, ConnectionError> {
        let buf = self.raw_reply_unchecked().await?;

        // Parse the reply
        let reply = buf.map(|buf| R::try_parse(buf.as_ref()).unwrap().0);
        Ok(reply)
    }
}

/// A cookie for a request that has a reply containing file descriptors.
#[derive(Debug)]
pub struct CookieWithFds<'conn, C: RequestConnection + ?Sized, R> {
    raw: RawCookie<'conn, C>,
    capture: PhantomData<R>,
}

impl<'conn, C: Connection + ?Sized, R: TryParseFd> CookieWithFds<'conn, C, R> {
    /// Create a new cookie from its raw parts.
    pub fn new(conn: &'conn C, sequence: SequenceNumber) -> Self {
        Self {
            raw: RawCookie::new(conn, sequence),
            capture: PhantomData,
        }
    }

    /// Get the sequence number of this cookie.
    pub fn sequence_number(&self) -> SequenceNumber {
        self.raw.sequence
    }

    /// Get the raw reply that the server sent.
    pub async fn raw_reply(self) -> Result<BufWithFds<C::Buf>, ReplyError> {
        // Wait for the reply
        let reply_or_error = self
            .raw
            .conn
            .wait_for_reply_with_fds_raw(self.raw.sequence)
            .await;

        // Wait until after the `await` to consume the cookie.
        let (conn, _) = self.raw.consume();

        // Check for errors
        match reply_or_error? {
            ReplyOrError::Reply(reply) => Ok(reply),
            ReplyOrError::Error(error) => Err(conn.parse_error(error.as_ref())?.into()),
        }
    }

    /// Get the reply that the server sent.
    pub async fn reply(self) -> Result<R, ReplyError> {
        let (buf, mut fds) = self.raw_reply().await?;

        // Parse the reply
        let (reply, _) = R::try_parse_fd(buf.as_ref(), &mut fds)?;
        Ok(reply)
    }
}

macro_rules! multiple_reply_cookie {
    (
        $(#[$outer:meta])*
        pub struct $name:ident for $reply:ident
    ) => {
        $(#[$outer])*
        pub struct $name<'conn, C> where C: RequestConnection + ?Sized {
            // The raw cookie we're polling.
            raw: Option<RawCookie<'conn, C>>,

            // Current wait future we're polling.
            wait: Option<Pin<Box<dyn Future<Output = Result<C::Buf, ReplyError>> + Send + 'conn>>>,
        }

        impl<'conn, C: RequestConnection + std::fmt::Debug + ?Sized> std::fmt::Debug for $name<'conn, C> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($name))
                    .field("raw", &self.raw)
                    .finish_non_exhaustive()
            }
        }

        impl<'conn, C: RequestConnection + ?Sized> $name<'conn, C> {
            pub(crate) fn new(
                cookie: Cookie<'conn, C, $reply>,
            ) -> Self {
                Self {
                    raw: Some(cookie.raw),
                    wait: None,
                }
            }

            /// Get the sequence number of this cookie.
            pub fn sequence_number(&self) -> Option<SequenceNumber> {
                self.raw.as_ref().map(|raw| raw.sequence)
            }
        }

        impl<C: RequestConnection + ?Sized> Stream for $name<'_, C> {
            type Item = Result<$reply, ReplyError>;

            fn poll_next(
                mut self: Pin<&mut Self>,
                cx: &mut Context<'_>,
            ) -> Poll<Option<Self::Item>> {
                loop {
                    // If we have a reply, try taking it.
                    if let Some(wait) = self.wait.as_mut() {
                        // Poll for the reply.
                        let reply = {
                            let raw_reply = match ready!(wait.as_mut().poll(cx)) {
                                Ok(reply) => reply,
                                Err(e) => return Poll::Ready(Some(Err(e))),
                            };

                            // Parse the reply.
                            match $reply::try_parse(raw_reply.as_ref()) {
                                Ok((reply, _)) => reply,
                                Err(e) => return Poll::Ready(Some(Err(e.into()))),
                            }
                        };

                        if Self::is_last(&reply) {
                            // Last one, end this stream.
                            self.wait = None;
                            self.raw = None;
                            return Poll::Ready(Some(Ok(reply)));
                        } else {
                            // More replies to come.
                            return Poll::Ready(Some(Ok(reply)));
                        }
                    }

                    // Take out the cookie.
                    let cookie = match self.raw.take() {
                        Some(cookie) => cookie,
                        None => return Poll::Ready(None),
                    };

                    // Begin waiting for a reply to this cookie.
                    self.wait = Some(
                        cookie.conn.wait_for_reply_or_error(cookie.sequence)
                    );
                    self.raw = Some(cookie);
                }
            }
        }
    };
}

multiple_reply_cookie!(
    /// A handle to the replies to a `ListFontsWithInfo` request.
    ///
    /// `ListFontsWithInfo` generated more than one reply, but `Cookie` only allows getting one reply.
    /// This structure implements `Iterator` and allows to get all the replies.
    pub struct ListFontsWithInfoCookie for ListFontsWithInfoReply
);

impl<C> ListFontsWithInfoCookie<'_, C>
where
    C: RequestConnection + ?Sized,
{
    fn is_last(reply: &ListFontsWithInfoReply) -> bool {
        reply.name.is_empty()
    }
}

#[cfg(feature = "record")]
multiple_reply_cookie!(
    /// A handle to the replies to a `record::EnableContext` request.
    ///
    /// `EnableContext` generated more than one reply, but `Cookie` only allows getting one reply.
    /// This structure implements `Iterator` and allows to get all the replies.
    pub struct RecordEnableContextCookie for EnableContextReply
);

#[cfg(feature = "record")]
impl<C> RecordEnableContextCookie<'_, C>
where
    C: RequestConnection + ?Sized,
{
    fn is_last(reply: &EnableContextReply) -> bool {
        // FIXME: There does not seem to be an enumeration of the category values, (value 5 is
        // EndOfData)
        reply.category == 5
    }
}
