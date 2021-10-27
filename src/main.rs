use blockchain_workshop::utils::get_bits_from_hash;

fn main() {
    dbg!(i32::from_str_radix(&"1d00ffff".to_string(), 16).unwrap());
    dbg!(get_bits_from_hash("00000000000000000015a35c0000000000000000000000000000000000000000".to_string()));
    dbg!(format!("{:2x}", 387294044));
}