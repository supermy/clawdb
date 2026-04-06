use byteorder::{ByteOrder, LittleEndian};
use rocksdb::merge_operator::MergeOperands;

pub fn create_vector_merge_operator(
) -> impl Fn(&[u8], Option<&[u8]>, &MergeOperands) -> Option<Vec<u8>> {
    move |_key: &[u8], existing_value: Option<&[u8]>, operands: &MergeOperands| {
        let mut result = existing_value.map(|v| v.to_vec()).unwrap_or_default();

        for operand in operands {
            if let Some(_existing) = existing_value {
                if !operand.is_empty() {
                    let update_type = operand[0];
                    match update_type {
                        0 => {
                            result = operand[1..].to_vec();
                        }
                        1 => {
                            if result.len() >= 4 && operand.len() >= 5 {
                                let offset = LittleEndian::read_u32(&operand[1..5]) as usize;
                                if offset + (operand.len() - 5) <= result.len() {
                                    result[offset..offset + operand.len() - 5]
                                        .copy_from_slice(&operand[5..]);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            } else {
                result = operand.to_vec();
            }
        }

        Some(result)
    }
}
