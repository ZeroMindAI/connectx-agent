use serde_json::Value;
use turbo_program::traits::TurboActionSerialization;

#[derive(Debug)]
pub enum GameAction {
    DropPiece(u8), // Column number (0-6) where to drop the piece
}

impl TurboActionSerialization for GameAction {
    fn deserialize(action: &[u8]) -> Result<(Self, &[u8]), &'static str> {
        let column = action[0];
        Ok((GameAction::DropPiece(column), &action[1..]))
    }

    fn serialize_json(json_str: &str) -> Result<Vec<u8>, &'static str> {
        let action: Value = serde_json::from_str(json_str).map_err(|_| "Invalid JSON")?;
        let mut result = Vec::new();

        if let Some(column) = action.as_u64().map(|n| n as u8) {
            result.push(column);
        } else {
            let action_type = action["action"].as_str().ok_or("Missing action field")?;
            let data = action["data"].as_array().ok_or("Missing data field")?;

            match action_type {
                "DropPiece" => {
                    if data.len() != 1 {
                        return Err("Invalid data length for DropPiece");
                    }
                    let column = data[0].as_u64().ok_or("Invalid column")? as u8;
                    if column >= 7 {
                        return Err("Column out of bounds");
                    }
                    result.push(column);
                }
                _ => return Err("Invalid action type"),
            }
        }

        Ok(result)
    }
}
