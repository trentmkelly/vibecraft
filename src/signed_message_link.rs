use crate::network::codec::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignedMessageLink {
    pub sender: Uuid,
    pub session_id: Uuid,
    pub index: i32,
}

impl SignedMessageLink {
    pub const MAX_INDEX: i32 = i32::MAX;
    pub const NIL_UUID: Uuid = Uuid([0; 16]);

    pub fn unsigned(sender: Uuid) -> Self {
        Self::root(sender, Self::NIL_UUID)
    }

    pub fn root(sender: Uuid, session_id: Uuid) -> Self {
        Self {
            sender,
            session_id,
            index: 0,
        }
    }

    pub fn update_signature(&self, output: &mut Vec<u8>) {
        output.extend_from_slice(&self.sender.0);
        output.extend_from_slice(&self.session_id.0);
        output.extend_from_slice(&self.index.to_be_bytes());
    }

    pub fn is_descendant_of(&self, link: &SignedMessageLink) -> bool {
        self.index > link.index && self.sender == link.sender && self.session_id == link.session_id
    }

    pub fn advance(&self) -> Option<Self> {
        if self.index == Self::MAX_INDEX {
            None
        } else {
            Some(Self {
                sender: self.sender,
                session_id: self.session_id,
                index: self.index + 1,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_message_link_matches_java_factories_signature_order_and_advancement() {
        let sender = Uuid([1; 16]);
        let session = Uuid([2; 16]);

        assert_eq!(
            SignedMessageLink::unsigned(sender),
            SignedMessageLink {
                sender,
                session_id: SignedMessageLink::NIL_UUID,
                index: 0,
            }
        );
        assert_eq!(
            SignedMessageLink::root(sender, session),
            SignedMessageLink {
                sender,
                session_id: session,
                index: 0,
            }
        );

        let link = SignedMessageLink {
            sender,
            session_id: session,
            index: 3,
        };
        let mut bytes = Vec::new();
        link.update_signature(&mut bytes);
        let mut expected = Vec::new();
        expected.extend_from_slice(&sender.0);
        expected.extend_from_slice(&session.0);
        expected.extend_from_slice(&3_i32.to_be_bytes());
        assert_eq!(bytes, expected);

        assert_eq!(
            link.advance(),
            Some(SignedMessageLink {
                sender,
                session_id: session,
                index: 4,
            })
        );
        assert_eq!(
            SignedMessageLink {
                sender,
                session_id: session,
                index: SignedMessageLink::MAX_INDEX,
            }
            .advance(),
            None
        );
    }

    #[test]
    fn signed_message_link_descendant_requires_same_sender_session_and_greater_index() {
        let sender = Uuid([1; 16]);
        let session = Uuid([2; 16]);
        let root = SignedMessageLink::root(sender, session);

        assert!(SignedMessageLink {
            sender,
            session_id: session,
            index: 1,
        }
        .is_descendant_of(&root));
        assert!(!SignedMessageLink {
            sender,
            session_id: session,
            index: 0,
        }
        .is_descendant_of(&root));
        assert!(!SignedMessageLink {
            sender: Uuid([9; 16]),
            session_id: session,
            index: 1,
        }
        .is_descendant_of(&root));
        assert!(!SignedMessageLink {
            sender,
            session_id: Uuid([8; 16]),
            index: 1,
        }
        .is_descendant_of(&root));
    }
}
