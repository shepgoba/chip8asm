use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::u16;

// 1: I, 2: DT, 3: ST
struct Instr {
	name: String,
	params: Vec<String>,
	refs_label: bool,
	has_immediate: bool,
	special_register: u8,
	special_register_src: bool,
	address: u16
}

struct Label {
	name: String,
	address: u16
}

struct Directive {
	name: String,
	dtype: u8,
	data: u16
}

struct Data {
	data: Vec<u8>,
	address: u16
}

fn is_label(word: &str) -> bool {
	!word.starts_with("@") && word.ends_with(":")
}

fn is_preprocessor(word: &str) -> bool {
	word.starts_with("@")
}

fn num_params_valid_for_instruction(num_params: usize, word: &str) -> bool {
	match word {
		"cls" | "ret" | "sys" => return num_params == 0,
		"jmp" => return num_params == 1 || num_params == 2,
		"call" | "skp" | "sknp" => return num_params == 1,
		"se" | "sne" | "shl" | "shr" | "ld" | "or" | "and" | "xor" | "add" | "sub" | "rnd" | "subn" => return num_params == 2,
		"drw" => return num_params == 3,
		_ => return false
	}
}

fn immediate_is_valid(imm: &str, max_size: u16) -> bool {

	if String::from(imm).starts_with("0x") {
		let no_prefix = imm.trim_start_matches("0x");
		let result = u16::from_str_radix(no_prefix, 16);
		return result.is_ok() && result.unwrap() <= max_size;
	} else if String::from(imm).starts_with("0b") {
		let no_prefix = imm.trim_start_matches("0b");
		let result = u16::from_str_radix(no_prefix, 2);
		return result.is_ok() && result.unwrap() <= max_size;
	}

	let result = u16::from_str_radix(imm, 10);
	result.is_ok() && result.unwrap() <= max_size
}

fn param_valid_for_instruction(param: &str, index: usize, instr: &mut Instr) -> bool {
	static REGISTER_TOKENS: [&str; 0x10] = ["v0", "v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8", "v9", "va", "vb", "vc", "vd", "ve", "vf"];
	static SPECIAL_REGISTER_TOKENS: [&str; 0x4] = ["I" , "dt", "st", "key"];
	match instr.name.as_str() {
		"jmp" => {
			if index == 0 {
				if param == REGISTER_TOKENS[0] {
					return true;
				}
				let is_immediate = immediate_is_valid(param, 0xfff);

				if !is_immediate {
					instr.refs_label = true;
					return true;
				}
				return is_immediate;
			} else if index == 1 {
				instr.has_immediate = true;
				return immediate_is_valid(param, 0xfff);
			}
			return false;
		},
		"call" => {
			if index == 0 {
				let is_immediate = immediate_is_valid(param, 0xfff);

				if !is_immediate {
					instr.refs_label = true;
					return true;
				}
				return is_immediate;
			}
			return false;
		},
		"rnd" => {
			if index == 0 {
				for token in REGISTER_TOKENS.iter() {
					if &param == token {
						return true;
					}
				}
				return false;
			} else if index == 1 {
				instr.has_immediate = true;
				return immediate_is_valid(param, 0xff);
			}

			return false;
		},
		"se" | "sne" | "add" => {
			if index == 0 {
				for token in REGISTER_TOKENS.iter() {
					if &param == token {
						return true;
					}
				}
				return false;
			} else if index == 1 {
				for token in REGISTER_TOKENS.iter() {
					if &param == token {
						return true;
					}
				}

				instr.has_immediate = true;
				return immediate_is_valid(param, 0xff);
			}
			return false;
		},
		"skp" | "sknp" => {
			if index == 0 {
				for token in REGISTER_TOKENS.iter() {
					if &param == token {
						return true;
					}
				}
			}
			return false;
		},
		"sub" | "xor" | "or" | "and" => {
			if index == 0 {
				for token in REGISTER_TOKENS.iter() {
					if &param == token {
						return true;
					}
				}
			} else if index == 1 {
				for token in REGISTER_TOKENS.iter() {
					if &param == token {
						return true;
					}
				}
			}
			return false;
		}
		"ld" => {
			if index == 0 {
				for token in REGISTER_TOKENS.iter() {
					if &param == token {
						return true;
					}
				}

				for (index, token) in SPECIAL_REGISTER_TOKENS.iter().enumerate() {
					if &param == token {
						instr.special_register = index as u8 + 1;
						instr.special_register_src = false;
						return true;
					}
				}
				return false;
			} else if index == 1 {
				for token in REGISTER_TOKENS.iter() {
					if &param == token {
						return true;
					}
				}


				if instr.special_register > 1 {
					return false;
				}


				// LD i, addr can be up to 0xfff
				if instr.special_register == 1 {
					instr.has_immediate = true;

					let is_immediate = immediate_is_valid(param, 0xfff);
					if !is_immediate {
						instr.refs_label = true;
						return true;
					}
					return is_immediate;
				} else {

					for (index, token) in SPECIAL_REGISTER_TOKENS.iter().enumerate() {
						if &param == token {
							instr.special_register = index as u8 + 1;
							instr.special_register_src = true;
							return true;
						}
					}
				}

				instr.has_immediate = true;
				return immediate_is_valid(param, 0xff);
			}
			return false;
		},
		"drw" => {
			if index == 0 {
				for token in REGISTER_TOKENS.iter() {
					if &param == token {
						return true;
					}
				}
			} else if index == 1 {
				for token in REGISTER_TOKENS.iter() {
					if &param == token {
						return true;
					}
				}
			} else if index == 2 {
				return immediate_is_valid(param, 0xf);
			}

			return false;
		}
		_ => return false
	}
}

/*
fn valid_tokens_for_param<'a>() -> Vec<&'a str> {
	vec!["cum!"]
}*/


fn num_for_string(hex: &str, force_hex: bool) -> u16 {
	if force_hex {
		return u16::from_str_radix(hex, 16).unwrap();
	}

	let has_hex_prefix = String::from(hex).starts_with("0x");
	if has_hex_prefix {
		let no_prefix = hex.trim_start_matches("0x");
		return u16::from_str_radix(no_prefix, 16).unwrap();
	}

	let has_binary_prefix = String::from(hex).starts_with("0b");
	if has_binary_prefix {
		let no_prefix = hex.trim_start_matches("0b");
		return u16::from_str_radix(no_prefix, 2).unwrap();
	}
	u16::from_str_radix(hex, 10).unwrap()
}

fn opcode_for_instruction(instr: &Instr, valid_labels: &Vec<Label>) -> u16 {
	match instr.name.as_str() {
		"cls" => return 0x00E0,
		"ret" => return 0x00EE,
		"sys" => return 0,
		"jmp" => {
			
			if instr.params.len() == 1 {
				if instr.refs_label {
					for label in valid_labels {
						if label.name == instr.params[0] {
							let addr = label.address;
							//println!("label address: {}", addr);
							return 0x1000 | (addr & 0x0fff);
						}
					}
					println!("Invalid label for instruction {}", instr.name);
					return 0xFFFF;
				} else {
					let addr = num_for_string(instr.params[0].as_str(), false);
					return 0x1000 | (addr & 0x0fff);
				}
			} else {
				let addr = num_for_string(instr.params[1].as_str(), false);
				return 0xB000 | (addr & 0x0fff);
			}
		},
		"call" => {
			if instr.refs_label {
				for label in valid_labels {
					if label.name == instr.params[0] {
						let addr = label.address;
						//println!("label address: {}", addr);
						return 0x2000 | (addr & 0x0fff);
					}
				}
				println!("Invalid label for instruction {}", instr.name);
				return 0xFFFF;
			} else {
				let addr = num_for_string(instr.params[0].as_str(), false);
				return 0x2000 | (addr & 0x0fff);
			}
		},
		"se" => {
			if instr.has_immediate {
				let no_register_prefix = &instr.params[0][1..2];
				let reg_num = num_for_string(no_register_prefix, true);
				let imm = num_for_string(&instr.params[1], false);
				return 0x3000 | (reg_num << 8) | (imm & 0xff);
			} else {
				let dest_reg = &instr.params[0][1..2];
				let dest_reg_num = num_for_string(dest_reg, true);

				let src_reg = &instr.params[1][1..2];
				let src_reg_num = num_for_string(src_reg, true);

				return 0x5000 | (dest_reg_num << 8) | (src_reg_num << 4);
			}
		},
		"sne" => {
			if instr.has_immediate {
				let no_register_prefix = &instr.params[0][1..2];
				let reg_num = num_for_string(no_register_prefix, true);
				let imm = num_for_string(&instr.params[1], false);
				return 0x4000 | (reg_num << 8) | (imm & 0xff);
			} else {
				let dest_reg = &instr.params[0][1..2];
				let dest_reg_num = num_for_string(dest_reg, true);

				let src_reg = &instr.params[1][1..2];
				let src_reg_num = num_for_string(src_reg, true);

				return 0x9000 | (dest_reg_num << 8) | (src_reg_num << 4);
			}
		},
		"skp" => {
			let no_register_prefix = &instr.params[0][1..2];
			let reg_num = num_for_string(no_register_prefix, true);
			return 0xE09E | (reg_num << 8);
		},
		"sknp" => {
			let no_register_prefix = &instr.params[0][1..2];
			let reg_num = num_for_string(no_register_prefix, true);
			return 0xE0A1 | (reg_num << 8);
		},
		"ld" => {
								///println!("hmm");
			//println!("type: {}\n", instr.special_register as u32);
			if instr.has_immediate {

				if instr.special_register > 0 {
					// I
					match instr.special_register {
						1 => {
							if instr.refs_label {
								for label in valid_labels {
									//println!("label name {}", label.name);
									if label.name == instr.params[1] {
										let addr = label.address;
										return 0xA000 | (addr & 0x0fff);
									}
								}
								println!("Invalid label for instruction {}", instr.name);
								return 0xFFFF;
							} else {
								let val = num_for_string(&instr.params[1], false);
								return 0xA000 | (val & 0x0fff);
							}
						},
						2 => {
							return 0;
						},
						3 => {
							return 0;
						}
						_ => return 0
					}
				} else {

					let no_register_prefix = &instr.params[0][1..2];
					let reg_num = num_for_string(no_register_prefix, true);
					let imm = num_for_string(&instr.params[1], false);
					return 0x6000 | (reg_num << 8) | (imm & 0xff);
				}
			} else {
				println!("hmm");
				if instr.special_register > 0 {
					if instr.special_register_src {
						if instr.special_register == 2 {
							let no_register_prefix = &instr.params[0][1..2];
							let reg_num = num_for_string(no_register_prefix, true);
							return 0xF007 | reg_num << 8;
						} else {
							let no_register_prefix = &instr.params[0][1..2];
							let reg_num = num_for_string(no_register_prefix, true);
							return 0xF00A | reg_num << 8;
						}
						return 0xFF;
					} else {
						if instr.special_register == 2 {
							let no_register_prefix = &instr.params[1][1..2];
							let reg_num = num_for_string(no_register_prefix, true);
							return 0xF015 | reg_num << 8;
						} else {
							let no_register_prefix = &instr.params[1][1..2];
							let reg_num = num_for_string(no_register_prefix, true);
							return 0xF018 | reg_num << 8;
						}

					}
				} else {
					let dest_reg = &instr.params[0][1..2];
					let dest_reg_num = num_for_string(dest_reg, true);

					let src_reg = &instr.params[1][1..2];
					let src_reg_num = num_for_string(src_reg, true);

					return 0x8000 | (dest_reg_num << 8) | (src_reg_num << 4);
				}
			}
		},
		"or" => {
			let dest_reg = &instr.params[0][1..2];
			let dest_reg_num = num_for_string(dest_reg, true);

			let src_reg = &instr.params[1][1..2];
			let src_reg_num = num_for_string(src_reg, true);

			return 0x8001 | (dest_reg_num << 8) | (src_reg_num << 4);
		},
		"and" => {
			let dest_reg = &instr.params[0][1..2];
			let dest_reg_num = num_for_string(dest_reg, true);

			let src_reg = &instr.params[1][1..2];
			let src_reg_num = num_for_string(src_reg, true);

			return 0x8002 | (dest_reg_num << 8) | (src_reg_num << 4);
		},
		"xor" => {
			let dest_reg = &instr.params[0][1..2];
			let dest_reg_num = num_for_string(dest_reg, true);

			let src_reg = &instr.params[1][1..2];
			let src_reg_num = num_for_string(src_reg, true);

			return 0x8003 | (dest_reg_num << 8) | (src_reg_num << 4);
		},
		"add" => {
			if instr.has_immediate {
				let no_register_prefix = &instr.params[0][1..2];

				let reg_num = num_for_string(no_register_prefix, true);
				let imm = num_for_string(&instr.params[1], false);
				return 0x7000 | (reg_num << 8) | (imm & 0xff);
			} else {
				let dest_reg = &instr.params[0][1..2];
				let dest_reg_num = num_for_string(dest_reg, true);

				let src_reg = &instr.params[1][1..2];
				let src_reg_num = num_for_string(src_reg, true);

				return 0x8004 | (dest_reg_num << 8) | (src_reg_num << 4);
			}
		},
		"rnd" => {
			let no_register_prefix = &instr.params[0][1..2];
			let reg_num = num_for_string(no_register_prefix, true);
			let imm = num_for_string(&instr.params[1], false);
			return 0xC000 | (reg_num << 8) | (imm & 0xff);
		},
		"drw" => {
			let reg_x_str = &instr.params[0][1..2];
			let reg_y_str = &instr.params[1][1..2];

			let reg_x = num_for_string(reg_x_str, true);
			let reg_y = num_for_string(reg_y_str, true);

			let num_bytes = num_for_string(&instr.params[2], false);

			return 0xD000 | (reg_x << 8) | (reg_y << 4) | (num_bytes & 0xf);
		}
		"sub" => {
			let dest_reg = &instr.params[0][1..2];
			let dest_reg_num = num_for_string(dest_reg, true);

			let src_reg = &instr.params[1][1..2];
			let src_reg_num = num_for_string(src_reg, true);

			return 0x8005 | (dest_reg_num << 8) | (src_reg_num << 4);
		}
		_ => return 0
	}
}

fn instruction_is_valid(word: &str) -> bool {
	static VALID_INSTRUCTIONS: [&str; 20] = ["cls", "ret", "sys", "jmp", "call", "se", "sne", 
											"ld", "or", "and", "xor", "add", "sub", "shr", 
											"subn", "shl", "rnd", "drw", "skp", "sknp"];
	for string in VALID_INSTRUCTIONS.iter() {
		// check if the instruction is valid
		if &word == string {
			return true;
		}
	}

	false
}

fn remove_suffix<'a>(s: &'a str, p: &str) -> &'a str {
    if s.ends_with(p) {
        return &s[..s.len() - p.len()];
    } else {
        s
    }
}

fn remove_prefix<'a>(s: &'a str, p: &str) -> &'a str {
	if s.starts_with(p) {
        &s[p.len()..]
    } else {
        s
    }
}

fn expected_params_for_directive(name: &str) -> u32 {
	match name {
		"org" => return 1,
		"db" => return 1,
		_ => 0
	}
}

use std::env;

fn main() {
	let args: Vec<String> = env::args().collect();
	if args.len() < 3 {
		println!("Usage: chip8asm <input file> <output file>"); 
		return;
	}

	let input = &args[1];
	let output = &args[2];

    let file_result = File::open(input);
    if !file_result.is_ok() {
    	println!("Error opening file {}", input);
    	return;
    }

    let file = file_result.unwrap();
    let reader = BufReader::new(file);

    let mut instrs: Vec<Instr> = Vec::new();
    let mut labels: Vec<Label> = Vec::new();
    let mut data: Vec<Data> = Vec::new();

    let mut current_address: u16 = 0x200;
    let mut max_rom_size: usize = 0;

    for (line_number, line) in reader.lines().enumerate() {
    	let current_line: String = line.unwrap();
    	let words: Vec<&str> = current_line.split_whitespace().collect();

    	if words.len() > 0 {

    		//comment
    		if words[0].starts_with(";") {
    			continue;
    		}
    		let mut word_offset = 0;

    		// add label to label list, possibly add instruction
    		if is_label(words[word_offset]) {
    			let new_label = Label {
					name: String::from(remove_suffix(words[word_offset], ":")),
    				address: current_address as u16
    			};
    			labels.push(new_label);

    			// no instructions after
    			if words.len() < 2 {
    				continue;
    			}

    			word_offset += 1;
    		} else if is_preprocessor(words[word_offset]) {
    			let directive_name = remove_prefix(words[word_offset], "@");

    			let mut new_directive = Directive {
    				name: String::from(directive_name),
    				dtype: 0, 
    				data: 0
    			};

    			let num_params_for_directive = expected_params_for_directive(directive_name);

    			let num_params = words.len() - word_offset - 1;
    			if num_params != num_params_for_directive as usize && directive_name != "db" {
    				println!("Invalid parameters for directive \"{}\" on line {}", directive_name, line_number + 1);
    				return;
    			}

    			if directive_name == "org" {
    				new_directive.dtype = 1;
    				new_directive.data = num_for_string(words[word_offset + 1], false);
    				if new_directive.data >= 0x1000 {
    					println!("Location beyond bounds");
    					return;
    				}
    				current_address = new_directive.data;

    			} else if directive_name == "db" {
    				new_directive.dtype = 2;

    				let mut new_data = Data {
    					data: Vec::new(),
    					address: current_address
    				};

    				for i in 1..num_params + 1 {
    					let num_str = String::from(words[i]);
    					let value: u8 = num_for_string(num_str.trim_end_matches(","), false) as u8;
    					new_data.data.push(value);
    				}

    				if num_params & 1 != 0 {
    					current_address += (num_params + 1) as u16;
    				} else {
    					current_address += num_params as u16;
    				}
    				
    				data.push(new_data);

    				if current_address as usize > max_rom_size {
    					max_rom_size = current_address as usize;
    				}
    			} else {
    				println!("Invalid directive \"{}\" on line {}", directive_name, line_number + 1);
    				return;
    			}
    			continue;
    		}
    		
    		let word = words[word_offset];
			if instruction_is_valid(word) {
				let mut new_instr = Instr {
					name: String::from(word),
					has_immediate: false,
					refs_label: false,
					params: Vec::new(),
					special_register: 0,
					special_register_src: false,
					address: current_address as u16
				};

				let num_params = words.len() - word_offset - 1;
				if num_params_valid_for_instruction(num_params, word) {
					for i in 0..num_params {
						let mut the_str = String::from(words[i + 1 + word_offset]);
						if !param_valid_for_instruction(the_str.trim_end_matches(","), i, &mut new_instr) {
							println!("Error: Invalid parameter {} on line {}... exiting", words[i + 1 +  word_offset], line_number + 1);
							return;
						}


						// enforce comma for parameters
						if i < num_params - 1 {
							if the_str.ends_with(",") {
								the_str.truncate(the_str.len() - 1);

							} else {
								println!("Error: Separate parameters on line {} with a comma", line_number + 1);
								return;
							}
						}


						new_instr.params.push(the_str);

					}
				} else {
					println!("Error: Invalid parameters on line {}", line_number + 1);
					return;
				}

				instrs.push(new_instr);
				current_address += 2;

			} else {
				println!("{} on line {} is not a valid instruction", word, line_number + 1);
				return;
			} 
    	}

    	if current_address as usize > max_rom_size {
    		max_rom_size = current_address as usize;
    	}
	}

	let mut byte_buffer: [u8; 0xE00] = [0; 0xE00];

	for dat in data {
		for index in 0..dat.data.len() {
			byte_buffer[(dat.address - 0x200 + index as u16) as usize] = dat.data[index];
		}
	}

    for instr in instrs {
		let opcode: u16 = opcode_for_instruction(&instr, &labels);

		//invalid opcode or error
		if opcode == 0xFFFF {
			return;
		}

		byte_buffer[(instr.address - 0x200) as usize] = (opcode >> 8) as u8;
		byte_buffer[(instr.address - 0x200 + 1) as usize] = (opcode & 0xff) as u8;
	}

	// write opcodes to file
	let mut buffer = File::create(output).expect("error opening output file");

	//while pos < max_rom_size {
    let bytes_written = buffer.write(&byte_buffer[0..(max_rom_size - 0x200)]);
    if bytes_written.unwrap() == 0 {
    	println!("Error writing output file");
    }

}