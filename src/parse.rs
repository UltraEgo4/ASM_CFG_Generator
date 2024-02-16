use std::fs::File;
use std::io::Read;
use std::path::Path;

use goblin::elf::Elf;
use iced_x86::{Decoder, DecoderOptions, FlowControl, Instruction};

use petgraph::dot::{Config, Dot};
use petgraph::stable_graph::NodeIndex;
use petgraph::Graph;

use self::graph::print_graph;

mod graph;

pub fn check_if_valid_elf_64(path: &Path) -> Result<(), String> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(err) => return Err(err.to_string()),
    };

    //read only the necessary bytes to check the ELF header
    //16 bytes to identify if its an elf
    let mut header = [0; 16];
    //if the result of the iflet is error execute code, if not an error go on
    if let Err(err) = file.read_exact(&mut header) {
        return Err(err.to_string());
    }
    //check if the file is an ELF file
    if header[..4] != [0x7F, b'E', b'L', b'F'] {
        return Err("Not an ELF file".to_string());
    }
    //check if its a 64-bit ELF file, 01 is 32
    if header[4] != 2 {
        return Err("Not 64-bit".to_string());
    }
    //next byte represent endianess 01 little endian, 02 big endian

    Ok(())
}

//TODO: reading only the nescessary parts of the binary
//load the whole binary into vec
pub fn load_binary(path: &Path) -> Result<Vec<u8>, String> {
    //check if valid elf before loading it

    //handle file open error
    //could use iflet here as well
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(err) => return Err(err.to_string()),
    };

    let mut buffer = Vec::new();

    //handle reading error, implicit return of Result<buffer(which is a vec),string (which is an error)>
    match file.read_to_end(&mut buffer) {
        //using Ok(_) means match any value and ignore it, if the result is Ok regardless of the value within Ok return Ok(buffer)
        Ok(_) => Ok(buffer),
        Err(err) => Err(err.to_string()),
    }

    //print elf metadata
    //println!("elf: {:#?}", &elf);
}

//print binary for debug purposes
pub fn print_binary(buffer: &[u8], offset: u64) {
    let usize_offset = offset as usize;
    //TODO: check if offset is in bounds

    for &byte in &buffer[usize_offset..] {
        //{:02X} format specifier, each byte is printed 2 chars wide and in hex
        print!("{:02X} ", byte);
    }
    println!();
}

//function to calculate the given virt_addrs location in the raw bytes
pub fn calculate_offset(buffer: &[u8], address: &u64) -> u64 {
    let parsed = Elf::parse(&buffer);

    //print elf metadata
    println!("elf: {:#?}", &parsed);

    //p_vaddr: starting virtual addr for the segment
    //p_offset: offset in the file where the segment begins

    //virtual_offset in segment determines where our address is in the segment = virtual addr - program_header.p_vaddr
    //file_offset the absolute offset from the start to the segment(where our address is in) + the offset from the segment start and our given addr = program_header.p_offset + virtual_offset
    let mut virtual_offset: u64 = 0;
    let mut file_offset: u64 = 0;

    //iterate thorugh the program headers
    //TODO:unwrap panics if parsed is error, handle parsed above
    for i in &parsed.unwrap().program_headers {
        //check if the given vaddr is between the start and end of the segment
        let segment_end = &i.p_vaddr + &i.p_memsz;
        if address >= &i.p_vaddr && address < &segment_end {
            virtual_offset = address - i.p_vaddr;
            file_offset = i.p_offset + virtual_offset;
            break;
        }
    }
    file_offset
}

pub fn calculate_segment_offset(buffer: &[u8], address: &u64) -> u64 {
    let parsed = Elf::parse(&buffer);
    //for i in &parsed.unwrap().program_headers {}
    return 5;
}

//this function returns a vec with a tuple of address, and Instruction
//the return contains all instructions in the function, use split_to_basic_blocks on it
pub fn reassemble(
    buffer: &[u8],
    file_offset: &u64,
    virtual_address: &u64,
) -> Vec<(u64, Instruction)> {
    let start_index = *file_offset as usize;
    let mut decoder = Decoder::with_ip(
        64,
        &buffer[start_index..], //changed from [start_index..] to &buffer start
        *virtual_address,
        DecoderOptions::NONE,
    );

    // Initialize this outside the loop because decode_out() writes to every field
    let mut instruction = Instruction::default();

    let mut assembly_line = Vec::new();

    while decoder.can_decode() {
        decoder.decode_out(&mut instruction);

        //instruction.ip is the address where the instruction is
        assembly_line.push((instruction.ip(), instruction));

        //break if instruction is Return type
        //if check is at the end so ret is included in the output
        if instruction.flow_control() == FlowControl::Return {
            break;
        }
    }

    //print vector for dbg purposes
    for (address, instruction) in &assembly_line {
        println!("{:016X}, {}", instruction.ip(), instruction);
    }

    assembly_line
}

//game loop function that contains every other function, this is called in main
pub fn generate_cfg(binary_path: &String, virtual_address: &u64) {
    //TODO: dont unwrap, send error upwards
    let _ = check_if_valid_elf_64(Path::new(binary_path)).unwrap();

    let binary = load_binary(Path::new(binary_path)).unwrap();

    let offset = calculate_offset(&binary, &virtual_address);

    let assembly_line = reassemble(&binary, &offset, &virtual_address);

    let mut graph = graph::Graph::new();

    graph = graph::split_to_basic_blocks(assembly_line, graph);

    graph = graph::determine_edges(graph);

    print_graph(&graph);

    create_dot(&graph);
}

//function to create the dot file
pub fn create_dot(graph: &graph::Graph) {
    let mut cfg = Graph::<_, ()>::new();

    for block in &graph.nodes {
        cfg.add_node(block.convert_assembly_line_to_string());
    }
    for edge in &graph.edges {
        cfg.add_edge(
            NodeIndex::new(edge.node_indexes.0 as usize),
            NodeIndex::new(edge.node_indexes.1 as usize),
            (),
        );
    }

    let dot_output = format!("{:?}", Dot::with_config(&cfg, &[Config::EdgeNoLabel]));

    let dot_output = dot_output.replace("\\n", "n");

    let dot_output = dot_output.replace(
        "{",
        "{
    node[
        shape=box,
    ]
    edge[
        tailport=s,
        headport=n,
    ]",
    );

    std::fs::write("output.dot", dot_output).expect("Failed to write Dot output to file");
}

/* node[
    shape=box,
]
edge[
    tailport=s,
    headport=n,
] */
