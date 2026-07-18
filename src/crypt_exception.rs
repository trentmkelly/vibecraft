//! Error wrapper matching Minecraft's checked `CryptException`.

#![allow(dead_code)]

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct CryptException {
    cause: Box<dyn Error + Send + Sync + 'static>,
}

impl CryptException {
    pub fn new(cause: impl Error + Send + Sync + 'static) -> Self {
        Self {
            cause: Box::new(cause),
        }
    }

    pub fn cause(&self) -> &(dyn Error + Send + Sync + 'static) {
        self.cause.as_ref()
    }
}

impl fmt::Display for CryptException {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.cause.fmt(formatter)
    }
}

impl Error for CryptException {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.cause.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::CryptException;
    use std::error::Error;

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn crypt_exception_matches_java_source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/CryptException.java");
        assert_eq!(JAVA.lines().count(), 7);
        for fragment in [
            "public class CryptException extends Exception",
            "public CryptException(final Throwable cause)",
            "super(cause)",
        ] {
            assert!(JAVA.contains(fragment), "missing CryptException source fragment: {fragment}");
        }
    }

    #[test]
    fn crypt_exception_retains_cause_and_error_chain() {
        let cause = std::io::Error::new(std::io::ErrorKind::InvalidData, "ciphertext");
        let error = CryptException::new(cause);
        assert_eq!(error.to_string(), "ciphertext");
        assert_eq!(error.cause().to_string(), "ciphertext");
        assert_eq!(error.source().expect("source is the Java cause").to_string(), "ciphertext");
    }
}
