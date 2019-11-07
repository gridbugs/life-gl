use nom::*;

fn from_hex(input: &str) -> Result<u8, ::std::num::ParseIntError> {
    u8::from_str_radix(input, 16)
}

fn is_hex_digit(c: char) -> bool {
    match c {
        '0'..='9' | 'a'..='f' | 'A'..='F' => true,
        _ => false,
    }
}

named!(hex_primary<&str, u8>, map_res!(take_while_m_n!(2, 2, is_hex_digit), from_hex));

named!(hex_colour<&str, [f32; 4]>,
    do_parse!(
        tag!("#") >>
        red: hex_primary >>
        green: hex_primary >>
        blue: hex_primary >>
    ([red as f32 / 255., green as f32 / 255., blue as f32 / 255., 1.])
    )
);

pub fn parse_colour(c: &str) -> Result<[f32; 4], String> {
    match hex_colour(c) {
        Ok((_, c)) => Ok(c),
        Err(e) => Err(format!("{}", e)),
    }
}
