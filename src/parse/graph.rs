use iced_x86::{FlowControl, Formatter, Instruction, NasmFormatter};
use std::collections::HashSet;
pub struct BasicBlock {
    pub id: u64,
    pub assembly_line: Vec<(u64, Instruction)>,
    pub start_addr: u64,
    pub end_addr: u64,
    pub jmp_addr: u64,
}

pub struct Edge {
    //index only, not basic blocks, i could modify it to that later, but its fine like this
    pub node_indexes: (u64, u64),
}

pub struct Graph {
    pub nodes: Vec<BasicBlock>,
    pub edges: Vec<Edge>,
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
}

impl BasicBlock {
    pub fn new() -> BasicBlock {
        BasicBlock {
            id: 0,
            assembly_line: Vec::new(),
            start_addr: 0,
            end_addr: 0,
            jmp_addr: 0,
        }
    }

    //method to add an addr+instruction combo to the assembly_line vector
    pub fn add_to_assembly_line(&mut self, address: u64, instruction: Instruction) {
        self.assembly_line.push((address, instruction));
    }

    //convert the addr and Instruction obj to a string to use it in the dot files label
    // and additional formatting for better output
    pub fn convert_assembly_line_to_string(&self) -> String {
        let mut assembly_line_string = String::new();
        let mut formatter = NasmFormatter::new();
        for (address, instruction) in &self.assembly_line {
            let address_str = format!("0x{:0X}", address);
            assembly_line_string += &address_str;
            assembly_line_string += " ";
            formatter.format(&instruction, &mut assembly_line_string);
            assembly_line_string.push('\n');
        }

        assembly_line_string
    }
}

/* pub fn split_to_basic_blocks(assembly_line: Vec<(u64, Instruction)>, mut graph: Graph) -> Graph {
    // save branch target address
    let mut jmp_targets = HashSet::new();

    //determine jmp targets and store them in the HashSet
    for (address, instruction) in &assembly_line {
        if instruction.flow_control() == FlowControl::UnconditionalBranch
            || instruction.flow_control() == FlowControl::ConditionalBranch
        {
            jmp_targets.insert(instruction.near_branch_target());
        }
    }

    //print out jmp targets for debug
    for i in &jmp_targets {
        println!("{}", i);
    }

    //construct basic blocks
    let mut current_block = BasicBlock::new();
    let mut splitted_blocks = Vec::new();
    let mut counter = 0;

    for (address, instruction) in assembly_line {
        match instruction.flow_control() {
            FlowControl::Next
            | FlowControl::Call
            | FlowControl::IndirectCall
            | FlowControl::Interrupt
            | FlowControl::XbeginXabortXend => {
                if current_block.start_addr == 0 {
                    current_block.start_addr = address;
                }

                //if current instrs address is in the hashset, create new block bcs we split here
                if jmp_targets.contains(&address) {
                    current_block.id = counter;

                    if current_block.end_addr == 0 {
                        current_block.end_addr = address;
                    }

                    splitted_blocks.push(current_block);

                    counter += 1;
                    current_block = BasicBlock::new();

                    current_block.start_addr = address;
                }
                current_block.end_addr = address;
                current_block.add_to_assembly_line(address, instruction);
            }
            //if anything else then next create new block
            //UnconditionalBranch,,ConditionalBranch
            FlowControl::UnconditionalBranch | FlowControl::ConditionalBranch => {
                //save target addr as a member variable as well
                current_block.jmp_addr = instruction.near_branch_target();
                current_block.add_to_assembly_line(address, instruction);

                //if jmp is the first element
                if current_block.start_addr == 0 {
                    current_block.start_addr = address;
                }
                current_block.end_addr = address;

                //if current instrs address is in the hashset (jmp is targeted by a jmp), and the next instrs is not
                //we check this because if theres an instr which is a target, and a jmp right after that
                //we would split 2 times thus creating an empty block
                if jmp_targets.contains(&instruction.near_branch_target())
                    && !jmp_targets.contains(&instruction.next_ip())
                {
                    current_block.id = counter;
                    if current_block.end_addr == 0 {
                        current_block.end_addr = address;
                    }
                    splitted_blocks.push(current_block);

                    counter += 1;
                    current_block = BasicBlock::new();
                }
            }
            //IndirectBranch,Return,Exception
            _ => {
                //anything else just push it, we only scanned the binary until ret, dont have to worry about anything else
                current_block.add_to_assembly_line(address, instruction);
            }
        };
    }

    //add last block
    if !current_block.assembly_line.is_empty() {
        current_block.id = counter;
        //set end address to the last instruction address
        current_block.end_addr = current_block.assembly_line.last().unwrap().0;
        splitted_blocks.push(current_block);
    }

    //debug print
    let mut formatter = NasmFormatter::new();
    let mut output = String::new();

    for (index, block) in splitted_blocks.iter().enumerate() {
        println!(
            "Block {}: start_addr: {:#X}, end_addr: {:#X}, jmp_addr: {:#X}",
            block.id, block.start_addr, block.end_addr, block.jmp_addr
        );
        println!("Assembly lines:");
        for (addr, instr) in &block.assembly_line {
            output.clear();
            formatter.format(&instr, &mut output);
            println!("Address: {:#X}, Instruction: {:?}", addr, &output);
        }
        println!();
    }

    graph.nodes = splitted_blocks;

    graph
} */

pub fn split_to_basic_blocks(assembly_line: Vec<(u64, Instruction)>, mut graph: Graph) -> Graph {
    // save branch target address
    let mut branch_target_leaders = HashSet::new();
    let mut after_branch_leaders = HashSet::new();

    // start leader
    let mut start_leader = assembly_line[0].0;

    //determine jmp targets and store them in the HashSet
    for (i, (address, instruction)) in assembly_line.iter().enumerate() {
        if instruction.flow_control() == FlowControl::UnconditionalBranch
            || instruction.flow_control() == FlowControl::ConditionalBranch
        {
            branch_target_leaders.insert(instruction.near_branch_target());
            after_branch_leaders.insert(instruction.next_ip());

            /* if assembly_line[i + 1].1.flow_control() != FlowControl::UnconditionalBranch
                || assembly_line[i + 1].1.flow_control() != FlowControl::ConditionalBranch
            {
                after_branch_leaders.insert(instruction.next_ip());
            } */
        }
    }

    //print out jmp targets for debug
    for i in &branch_target_leaders {
        println!("{:0X}", i);
    }
    println!("---------------");
    for i in &after_branch_leaders {
        println!("{:0X}", i);
    }
    println!("---------------");
    println!("{:0X}", start_leader);

    //construct basic blocks
    let mut current_block = BasicBlock::new();
    let mut splitted_blocks = Vec::new();
    let mut counter = 0;

    for (address, instruction) in assembly_line {
        match instruction.flow_control() {
            FlowControl::Next
            | FlowControl::Call
            | FlowControl::IndirectCall
            | FlowControl::Interrupt
            | FlowControl::XbeginXabortXend => {
                if current_block.start_addr == 0 {
                    current_block.start_addr = address;
                }

                //if current instrs address is in the hashset, create new block bcs we split here
                if branch_target_leaders.contains(&address) {
                    current_block.id = counter;

                    if current_block.end_addr == 0 {
                        current_block.end_addr = address;
                    }

                    splitted_blocks.push(current_block);

                    counter += 1;
                    current_block = BasicBlock::new();

                    current_block.start_addr = address;
                }
                current_block.end_addr = address;
                current_block.add_to_assembly_line(address, instruction);
            }
            //if anything else then next create new block
            //UnconditionalBranch,,ConditionalBranch
            FlowControl::UnconditionalBranch | FlowControl::ConditionalBranch => {
                //save target addr as a member variable as well
                current_block.jmp_addr = instruction.near_branch_target();
                current_block.add_to_assembly_line(address, instruction);

                //if jmp is the first element
                if current_block.start_addr == 0 {
                    current_block.start_addr = address;
                }
                current_block.end_addr = address;

                //if current instrs address is in the hashset (jmp is targeted by a jmp), and the next instrs is not
                //we check this because if theres an instr which is a target, and a jmp right after that
                //we would split 2 times thus creating an empty block
                if branch_target_leaders.contains(&instruction.near_branch_target())
                    && !branch_target_leaders.contains(&instruction.next_ip())
                {
                    current_block.id = counter;
                    if current_block.end_addr == 0 {
                        current_block.end_addr = address;
                    }
                    splitted_blocks.push(current_block);

                    counter += 1;
                    current_block = BasicBlock::new();
                }
            }
            //IndirectBranch,Return,Exception
            _ => {
                //anything else just push it, we only scanned the binary until ret, dont have to worry about anything else
                current_block.add_to_assembly_line(address, instruction);
            }
        };
    }

    //add last block
    if !current_block.assembly_line.is_empty() {
        current_block.id = counter;
        //set end address to the last instruction address
        current_block.end_addr = current_block.assembly_line.last().unwrap().0;
        splitted_blocks.push(current_block);
    }

    //debug print
    let mut formatter = NasmFormatter::new();
    let mut output = String::new();

    for (index, block) in splitted_blocks.iter().enumerate() {
        println!(
            "Block {}: start_addr: {:#X}, end_addr: {:#X}, jmp_addr: {:#X}",
            block.id, block.start_addr, block.end_addr, block.jmp_addr
        );
        println!("Assembly lines:");
        for (addr, instr) in &block.assembly_line {
            output.clear();
            formatter.format(&instr, &mut output);
            println!("Address: {:#X}, Instruction: {:?}", addr, &output);
        }
        println!();
    }

    graph.nodes = splitted_blocks;

    graph
}

//little monkey

pub fn determine_edges(mut graph: Graph) -> Graph {
    //create edges between consecutive blocks
    for i in 0..graph.nodes.len() - 1 {
        graph.edges.push(Edge {
            node_indexes: (graph.nodes[i].id, graph.nodes[i + 1].id),
        });
    }

    //create edges between jmp target blocks
    for block in &graph.nodes {
        //jmp addr is set 0 if not cond or uncond jmps in split to blocks
        if block.jmp_addr != 0 {
            //iterate through the graph nodes and uses position to take a closure where we check a condition
            //and if thats true we create and push the edge into the graph
            //couldve done this with a for loop and if, but this is for practicing
            if let Some(target_block_index) = graph.nodes.iter().position(|other_block| {
                block.jmp_addr >= other_block.start_addr && block.jmp_addr <= other_block.end_addr
            }) {
                let edge = Edge {
                    node_indexes: (block.id, graph.nodes[target_block_index].id),
                };
                graph.edges.push(edge);
            }
        }
    }
    graph
}

pub fn print_graph(graph: &Graph) {
    println!("Nodes:");
    let mut assembly_line = String::new();
    let mut formatter = NasmFormatter::new();
    for block in &graph.nodes {
        println!(
            "ID: {}, Start Address: {:X}, End Address: {:X}, Jump Address: {:X}",
            block.id, block.start_addr, block.end_addr, block.jmp_addr
        );
        println!("Assembly Line:");
        println!("{}", block.convert_assembly_line_to_string());
        println!("-------------------------");
    }
    println!("Edges:");
    for edge in &graph.edges {
        println!("Edge: {:?}", edge.node_indexes);
    }
}
