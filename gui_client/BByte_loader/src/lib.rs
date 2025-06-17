


pub fn rot13_in_place(buf: &mut [u8]) {
    for byte in buf.iter_mut() {
        match *byte {
            b'a'..=b'm' | b'A'..=b'M' => *byte += 13, 
            b'n'..=b'z' | b'N'..=b'Z' => *byte -= 13,  
            _ => {}  
        }
    }
}