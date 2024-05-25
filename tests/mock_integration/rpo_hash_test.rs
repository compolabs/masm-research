use miden_vm::{prove, verify, Assembler, DefaultHost, ProvingOptions, StackInputs};
use miden_objects::{
  notes::
      NoteInputs,
  Felt, Hasher, Digest
};
#[test]
fn test_rpo_hash() {
    // Instantiate the assembler
    let assembler = Assembler::default().with_debug_mode(true);

    // Read the assembly program from a file
    let assembly_code: &str = include_str!("../../src/hash/rpo_test.masm");

    // Compile the program from the loaded assembly code
    let program = assembler
        .compile(assembly_code)
        .expect("Failed to compile the assembly code");

    let stack_inputs = StackInputs::try_from_ints([]).unwrap();
    let cloned_inputs = stack_inputs.clone();

    let host = DefaultHost::default();

    // Execute the program and generate a STARK proof
    let (outputs, proof) = prove(&program, stack_inputs, host, ProvingOptions::default())
        .expect("Failed to execute the program and generate a proof");

    println!("Stack output:");
    println!("{:?}", outputs.stack());

    let inputs = NoteInputs::new(vec![Felt::new(2)]).unwrap();

    println!("Inputs Hash: {:?}", inputs.commitment());

    let serial_num = [Felt::new(1), Felt::new(2), Felt::new(3), Felt::new(4)];
    let serial_num_hash = Hasher::merge(&[serial_num.into(), Digest::default()]);

    println!("Serial Number Hash: {:?}", serial_num_hash);

    verify(program.into(), cloned_inputs, outputs, proof).unwrap();
    println!("Program run successfully");
}
