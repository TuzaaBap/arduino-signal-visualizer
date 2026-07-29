use crate::ProtocolError;

pub fn encode(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len() + 2);
    output.push(0);
    let mut code_index = 0;
    let mut code = 1_u8;

    for &byte in input {
        if byte == 0 {
            output[code_index] = code;
            code_index = output.len();
            output.push(0);
            code = 1;
        } else {
            output.push(byte);
            code = code.wrapping_add(1);
            if code == 0xff {
                output[code_index] = code;
                code_index = output.len();
                output.push(0);
                code = 1;
            }
        }
    }

    output[code_index] = code;
    output
}

pub fn decode(input: &[u8]) -> Result<Vec<u8>, ProtocolError> {
    let mut output = Vec::with_capacity(input.len());
    let mut index = 0;

    while index < input.len() {
        let code = input[index] as usize;
        if code == 0 {
            return Err(ProtocolError::MalformedCobs);
        }
        index += 1;
        let next = index + code - 1;
        if next > input.len() {
            return Err(ProtocolError::MalformedCobs);
        }
        output.extend_from_slice(&input[index..next]);
        index = next;
        if code != 0xff && index < input.len() {
            output.push(0);
        }
    }

    Ok(output)
}
