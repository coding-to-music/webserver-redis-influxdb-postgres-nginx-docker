pub use connect::*;
pub use make_move::*;

mod connect {
    use std::convert::TryFrom;

    #[derive(serde::Serialize, Clone, Debug)]
    pub struct ConnectToGameParams {
        room_id: String,
    }

    impl ConnectToGameParams {
        fn new(room_id: String) -> Self {
            Self { room_id }
        }

        pub fn room_id(&self) -> &str {
            &self.room_id
        }
    }

    #[derive(serde::Deserialize)]
    struct ConnectToGameParamsBuilder {
        room_id: String,
    }

    impl ConnectToGameParamsBuilder {
        fn build(self) -> Result<ConnectToGameParams, ConnectToGameParamsInvalid> {
            Ok(ConnectToGameParams {
                room_id: self.room_id,
            })
        }
    }

    impl TryFrom<crate::JsonRpcRequest> for ConnectToGameParams {
        type Error = ConnectToGameParamsInvalid;

        fn try_from(request: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
            let builder: ConnectToGameParamsBuilder = serde_json::from_value(request.params)
                .map_err(ConnectToGameParamsInvalid::InvalidFormat)?;

            builder.build()
        }
    }

    #[derive(Debug)]
    pub enum ConnectToGameParamsInvalid {
        InvalidFormat(serde_json::Error),
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct ConnectToGameResult {
        connected: bool,
    }
}

mod make_move {
    use std::convert::TryFrom;

    use chess::{ChessBoard, ChessMove};

    #[derive(serde::Serialize, Clone, Debug)]
    pub struct MakeMoveParams {
        room_id: String,
        chess_move: ChessMove,
    }

    impl MakeMoveParams {
        fn new(room_id: String, chess_move: ChessMove) -> Self {
            Self {
                room_id,
                chess_move,
            }
        }

        pub fn room_id(&self) -> &str {
            &self.room_id
        }
    }

    #[derive(serde::Deserialize)]
    struct MakeMoveParamsBuilder {
        room_id: String,
        chess_move: ChessMove,
    }

    impl MakeMoveParamsBuilder {
        fn build(self) -> Result<MakeMoveParams, MakeMoveParamsInvalid> {
            Ok(MakeMoveParams {
                room_id: self.room_id,
                chess_move: self.chess_move,
            })
        }
    }

    impl TryFrom<crate::JsonRpcRequest> for MakeMoveParams {
        type Error = MakeMoveParamsInvalid;

        fn try_from(request: crate::JsonRpcRequest) -> Result<Self, Self::Error> {
            let builder: MakeMoveParamsBuilder = serde_json::from_value(request.params)
                .map_err(MakeMoveParamsInvalid::InvalidFormat)?;

            builder.build()
        }
    }

    #[derive(Debug)]
    pub enum MakeMoveParamsInvalid {
        InvalidFormat(serde_json::Error),
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct MakeMoveResult {
        board: ChessBoard,
    }
}
