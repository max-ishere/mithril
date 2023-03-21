extern crate blake2b_simd;
extern crate mithril;

use self::blake2b_simd::blake2b;
use mithril::byte_string::{string_to_u8_array, u8_array_to_string};
use mithril::randomx::common::randomx_reciprocal;
use mithril::randomx::hash::gen_program_aes_4rx4;
use mithril::randomx::m128::m128d;
use mithril::randomx::memory::VmMemory;
use mithril::randomx::program::{
    a_reg, e_reg, f_reg, r_reg, Instr, Mode, Opcode, Program, Store, REG_NEEDS_DISPLACEMENT,
    REG_NEEDS_DISPLACEMENT_IX,
};
use mithril::randomx::vm::{hash_to_m128i_array, new_register, new_vm, Vm};
use std::sync::Arc;

#[allow(overflowing_literals)]
const IMM32: i32 = 0xc0cb96d2; //3234567890
const IMM64: u64 = 0xffffffffc0cb96d2;
const ROUND_TO_NEAREST: u32 = 0;
const ROUND_DOWN: u32 = 1;
const ROUND_UP: u32 = 2;
const ROUND_TO_ZERO: u32 = 3;

#[test]
fn test_calculate_hash_1_with_light_memory() {
    let mut vm = new_vm(Arc::new(VmMemory::light(b"test key 000")));
    let result = vm.calculate_hash(b"This is a test");
    assert_eq!(
        "639183aae1bf4c9a35884cb46b09cad9175f04efd7684e7262a0ac1c2f0b4e3f",
        u8_array_to_string(result.as_bytes())
    );

    let result = vm.calculate_hash(b"Lorem ipsum dolor sit amet");
    assert_eq!(
        "300a0adb47603dedb42228ccb2b211104f4da45af709cd7547cd049e9489c969",
        u8_array_to_string(result.as_bytes())
    );

    let result =
        vm.calculate_hash(b"sed do eiusmod tempor incididunt ut labore et dolore magna aliqua");
    assert_eq!(
        "c36d4ed4191e617309867ed66a443be4075014e2b061bcdaf9ce7b721d2b77a8",
        u8_array_to_string(result.as_bytes())
    );
}

#[test]
fn test_calculate_hash_1_with_full_memory() {
    let mut vm = new_vm(Arc::new(VmMemory::full(b"test key 000")));
    let result = vm.calculate_hash(b"This is a test");
    assert_eq!(
        "639183aae1bf4c9a35884cb46b09cad9175f04efd7684e7262a0ac1c2f0b4e3f",
        u8_array_to_string(result.as_bytes())
    );

    let result = vm.calculate_hash(b"Lorem ipsum dolor sit amet");
    assert_eq!(
        "300a0adb47603dedb42228ccb2b211104f4da45af709cd7547cd049e9489c969",
        u8_array_to_string(result.as_bytes())
    );

    let result =
        vm.calculate_hash(b"sed do eiusmod tempor incididunt ut labore et dolore magna aliqua");
    assert_eq!(
        "c36d4ed4191e617309867ed66a443be4075014e2b061bcdaf9ce7b721d2b77a8",
        u8_array_to_string(result.as_bytes())
    );
}

#[test]
fn test_calculate_hash_2_with_light_memory() {
    let mut vm = new_vm(Arc::new(VmMemory::light(b"test key 001")));

    let result =
        vm.calculate_hash(b"sed do eiusmod tempor incididunt ut labore et dolore magna aliqua");
    assert_eq!(
        "e9ff4503201c0c2cca26d285c93ae883f9b1d30c9eb240b820756f2d5a7905fc",
        u8_array_to_string(result.as_bytes())
    );

    let seed = string_to_u8_array("0b0b98bea7e805e0010a2126d287a2a0cc833d312cb786385a7c2f9de69d25537f584a9bc9977b00000000666fd8753bf61a8631f12984e3fd44f4014eca629276817b56f32e9b68bd82f416");
    let result = vm.calculate_hash(&seed);
    assert_eq!(
        "c56414121acda1713c2f2a819d8ae38aed7c80c35c2a769298d34f03833cd5f1",
        u8_array_to_string(result.as_bytes())
    );
}

#[test]
fn test_calculate_hash_2_with_full_memory() {
    let mut vm = new_vm(Arc::new(VmMemory::full(b"test key 001")));

    let result =
        vm.calculate_hash(b"sed do eiusmod tempor incididunt ut labore et dolore magna aliqua");
    assert_eq!(
        "e9ff4503201c0c2cca26d285c93ae883f9b1d30c9eb240b820756f2d5a7905fc",
        u8_array_to_string(result.as_bytes())
    );

    let seed = string_to_u8_array("0b0b98bea7e805e0010a2126d287a2a0cc833d312cb786385a7c2f9de69d25537f584a9bc9977b00000000666fd8753bf61a8631f12984e3fd44f4014eca629276817b56f32e9b68bd82f416");
    let result = vm.calculate_hash(&seed);
    assert_eq!(
        "c56414121acda1713c2f2a819d8ae38aed7c80c35c2a769298d34f03833cd5f1",
        u8_array_to_string(result.as_bytes())
    );
}

//Bugfix Test
#[test]
fn test_calculate_hash_3_with_full_memory() {
    let mut vm = new_vm(Arc::new(VmMemory::full(&string_to_u8_array(
        "15564c3122550436919ac2f8a71baf7cbaf9a4117b842d7f2b19dfd27dd178e9",
    ))));

    let seed = string_to_u8_array("0e0e8bb48b8406bf43039198b7712a35031e0607036ebf9afb3096977e7b8fb88c751430e96b02000006ad82bd221c5e282d0533c5dcca38f30babc2e62cd3aa03a965f8aec8ad6f129f5211");
    let result = vm.calculate_hash(&seed);
    assert_eq!(
        "312a2ef18681e7b065f87e56b2627f0a11e19b30415314efa898a13f407f5d08",
        u8_array_to_string(result.as_bytes())
    );
}

#[test]
fn test_init_scratchpad() {
    let mut vm = new_test_vm();
    let hash = blake2b("This is a test".as_bytes());
    vm.init_scratchpad(&hash_to_m128i_array(&hash));
    //sample test scratchpad layout
    assert_eq!(vm.scratchpad[0], 0x45a1b4e3e7fea6c);
    assert_eq!(vm.scratchpad[1], 0xe287d43cd65fd299);
    assert_eq!(vm.scratchpad[2], 0xbb1f8ec38ad6bcef);
    assert_eq!(vm.scratchpad[3], 0xc138a9a5c95e7b0f);
    assert_eq!(vm.scratchpad[4], 0x5cb93a85f06ef6e8);
    assert_eq!(vm.scratchpad[5], 0x6db2f212bf8390f8);
    assert_eq!(vm.scratchpad[6], 0x742a671fe69f28ab);
    assert_eq!(vm.scratchpad[7], 0xd6eb5539a8b4e48f);

    assert_eq!(vm.scratchpad[33333], 0x5b85caaea16199bf);
    assert_eq!(vm.scratchpad[66666], 0x3b35256a8a5afc64);
    assert_eq!(vm.scratchpad[131071], 0xc87ac0bce6ef30e8);
    assert_eq!(vm.scratchpad[191000], 0xf5e560770bdd6a4f);
    assert_eq!(vm.scratchpad[262142], 0x2e417916bf21fc05);
    assert_eq!(vm.scratchpad[262143], 0x66db274303c4fd4);
}

#[test]
fn test_init_vm() {
    let mut vm = new_test_vm();

    let hash = blake2b("This is a test".as_bytes());
    let seed = hash_to_m128i_array(&hash);
    let seed = vm.init_scratchpad(&seed);
    let prog = Program::from_bytes(gen_program_aes_4rx4(&seed, 136));
    vm.init_vm(&prog);

    assert_eq!(
        vm.reg.a[0].as_u64(),
        (0x4019c856c26708a9, 0x418e4a297ebfc304)
    );
    assert_eq!(
        vm.reg.a[1].as_u64(),
        (0x41e807a5dc7740b5, 0x40cd8725df13238a)
    );
    assert_eq!(
        vm.reg.a[2].as_u64(),
        (0x417112c274f91d68, 0x4176971a789beed7)
    );
    assert_eq!(
        vm.reg.a[3].as_u64(),
        (0x40bd229eeedd8e98, 0x414e441747df76c6)
    );

    assert_eq!(vm.config.e_mask[0], 0x3c000000001e145f);
    assert_eq!(vm.config.e_mask[1], 0x3a0000000011d432);

    assert_eq!(vm.config.read_reg[0], 0);
    assert_eq!(vm.config.read_reg[1], 3);
    assert_eq!(vm.config.read_reg[2], 5);
    assert_eq!(vm.config.read_reg[3], 7);

    assert_eq!(vm.mem_reg.ma, 0x738ddb40);
    assert_eq!(vm.mem_reg.mx, 0x8a8a6230);
}

#[test]
fn test_register_to_bytes() {
    let mut reg = new_register();
    reg.r[0] = 0x34ffebd12d810880;
    reg.r[1] = 0x6a80260a6208adef;
    reg.r[2] = 0x4f5d1008ee3b292f;
    reg.r[3] = 0xb65180d5769c17d0;
    reg.r[4] = 0x2695aed734fdb28;
    reg.r[5] = 0x3c6a84d4c01ddff5;
    reg.r[6] = 0xa9d93cadfd06d699;
    reg.r[7] = 0xc8ae2f0947643d9;
    reg.f[0] = m128d::from_u64(0x8436536b210b2639, 0x856723cf061d0955);
    reg.f[1] = m128d::from_u64(0x8327712703bab8b8, 0xfe381f5303432413);
    reg.f[2] = m128d::from_u64(0x9213c07a21421d21, 0x928b31fb36ecba0a);
    reg.f[3] = m128d::from_u64(0x82886513418828bb, 0x7ebb3de2ae60d7f4);
    reg.e[0] = m128d::from_u64(0x45ea59134401e457, 0x44850ec11d8a94c7);
    reg.e[1] = m128d::from_u64(0x428870e600b31bd8, 0x3fea167cf9422f28);
    reg.e[2] = m128d::from_u64(0x53dd0cedf7e2d75e, 0x53c16a0c2972cc15);
    reg.e[3] = m128d::from_u64(0x4379fad7dcb15a7d, 0x3f6980958c0ab574);
    reg.a[0] = m128d::from_u64(0xd14dcee38dfdc313, 0x452bdbf00bb500dc);
    reg.a[1] = m128d::from_u64(0x863af2ea80c745a7, 0x3be75a066e67b2e3);
    reg.a[2] = m128d::from_u64(0x94ff8c6994073d88, 0xdc24859a54929d04);
    reg.a[3] = m128d::from_u64(0xe725aa19567fa59c, 0x4b7f3597f285ef34);

    let bytes = reg.to_bytes();

    assert_eq!(
        bytes,
        [
            0x80, 0x08, 0x81, 0x2d, 0xd1, 0xeb, 0xff, 0x34, 0xef, 0xad, 0x08, 0x62, 0x0a, 0x26,
            0x80, 0x6a, 0x2f, 0x29, 0x3b, 0xee, 0x08, 0x10, 0x5d, 0x4f, 0xd0, 0x17, 0x9c, 0x76,
            0xd5, 0x80, 0x51, 0xb6, 0x28, 0xdb, 0x4f, 0x73, 0xed, 0x5a, 0x69, 0x02, 0xf5, 0xdf,
            0x1d, 0xc0, 0xd4, 0x84, 0x6a, 0x3c, 0x99, 0xd6, 0x06, 0xfd, 0xad, 0x3c, 0xd9, 0xa9,
            0xd9, 0x43, 0x76, 0x94, 0xf0, 0xe2, 0x8a, 0x0c, 0x55, 0x09, 0x1d, 0x06, 0xcf, 0x23,
            0x67, 0x85, 0x39, 0x26, 0x0b, 0x21, 0x6b, 0x53, 0x36, 0x84, 0x13, 0x24, 0x43, 0x03,
            0x53, 0x1f, 0x38, 0xfe, 0xb8, 0xb8, 0xba, 0x03, 0x27, 0x71, 0x27, 0x83, 0x0a, 0xba,
            0xec, 0x36, 0xfb, 0x31, 0x8b, 0x92, 0x21, 0x1d, 0x42, 0x21, 0x7a, 0xc0, 0x13, 0x92,
            0xf4, 0xd7, 0x60, 0xae, 0xe2, 0x3d, 0xbb, 0x7e, 0xbb, 0x28, 0x88, 0x41, 0x13, 0x65,
            0x88, 0x82, 0xc7, 0x94, 0x8a, 0x1d, 0xc1, 0x0e, 0x85, 0x44, 0x57, 0xe4, 0x01, 0x44,
            0x13, 0x59, 0xea, 0x45, 0x28, 0x2f, 0x42, 0xf9, 0x7c, 0x16, 0xea, 0x3f, 0xd8, 0x1b,
            0xb3, 0x00, 0xe6, 0x70, 0x88, 0x42, 0x15, 0xcc, 0x72, 0x29, 0x0c, 0x6a, 0xc1, 0x53,
            0x5e, 0xd7, 0xe2, 0xf7, 0xed, 0x0c, 0xdd, 0x53, 0x74, 0xb5, 0x0a, 0x8c, 0x95, 0x80,
            0x69, 0x3f, 0x7d, 0x5a, 0xb1, 0xdc, 0xd7, 0xfa, 0x79, 0x43, 0xdc, 0x00, 0xb5, 0x0b,
            0xf0, 0xdb, 0x2b, 0x45, 0x13, 0xc3, 0xfd, 0x8d, 0xe3, 0xce, 0x4d, 0xd1, 0xe3, 0xb2,
            0x67, 0x6e, 0x06, 0x5a, 0xe7, 0x3b, 0xa7, 0x45, 0xc7, 0x80, 0xea, 0xf2, 0x3a, 0x86,
            0x04, 0x9d, 0x92, 0x54, 0x9a, 0x85, 0x24, 0xdc, 0x88, 0x3d, 0x07, 0x94, 0x69, 0x8c,
            0xff, 0x94, 0x34, 0xef, 0x85, 0xf2, 0x97, 0x35, 0x7f, 0x4b, 0x9c, 0xa5, 0x7f, 0x56,
            0x19, 0xaa, 0x25, 0xe7,
        ]
    );
}

#[test]
fn test_exec_iadd_rs() {
    let instr = Instr {
        op: Opcode::IADD_RS,
        dst: r_reg(0),
        src: r_reg(1),
        imm: None,
        unsigned_imm: false,
        mode: Mode::Shft(3),
        target: None,
        effect: Vm::exec_iadd_rs,
    };
    let mut vm = new_test_vm();
    vm.reg.r[0] = 0x8000000000000000;
    vm.reg.r[1] = 0x1000000000000000;
    instr.execute(&mut vm);
    assert_eq!(vm.reg.r[0], 0x0);
}

#[test]
fn test_exec_iadd_rs_with_immediate() {
    let instr = Instr {
        op: Opcode::IADD_RS,
        dst: REG_NEEDS_DISPLACEMENT,
        src: r_reg(1),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::Shft(2),
        target: None,
        effect: Vm::exec_iadd_rs,
    };
    let mut vm = new_test_vm();
    vm.reg.r[REG_NEEDS_DISPLACEMENT_IX] = 0x8000000000000000;
    vm.reg.r[1] = 0x2000000000000000;
    instr.execute(&mut vm);
    assert_eq!(vm.reg.r[REG_NEEDS_DISPLACEMENT_IX], IMM64);
}

#[test]
#[allow(overflowing_literals)]
fn test_exec_isub_r() {
    let instr = Instr {
        op: Opcode::ISUB_R,
        dst: r_reg(0),
        src: r_reg(1),
        imm: None,
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_isub_r,
    };
    let mut vm = new_test_vm();
    vm.reg.r[0] = 1;
    vm.reg.r[1] = 0xFFFFFFFF;
    instr.execute(&mut vm);
    assert_eq!(vm.reg.r[0], 0xFFFFFFFF00000002);
}

#[test]
fn test_exec_isub_r_with_immediate() {
    let instr = Instr {
        op: Opcode::ISUB_R,
        dst: r_reg(0),
        src: Store::NONE,
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_isub_r,
    };
    let mut vm = new_test_vm();
    vm.reg.r[0] = 0;
    instr.execute(&mut vm);
    assert_eq!(vm.reg.r[0], (!IMM64 + 1));
}

#[test]
fn test_exec_imul_r() {
    let instr = Instr {
        op: Opcode::IMUL_R,
        dst: r_reg(0),
        src: r_reg(1),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_imul_r,
    };
    let mut vm = new_test_vm();
    vm.reg.r[0] = 0xBC550E96BA88A72B;
    vm.reg.r[1] = 0xF5391FA9F18D6273;

    instr.execute(&mut vm);
    assert_eq!(vm.reg.r[0], 0x28723424A9108E51);
}

#[test]
fn test_exec_imul_r_with_immediate() {
    let instr = Instr {
        op: Opcode::IMUL_R,
        dst: r_reg(0),
        src: Store::NONE,
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_imul_r,
    };
    let mut vm = new_test_vm();
    vm.reg.r[0] = 1;

    instr.execute(&mut vm);
    assert_eq!(vm.reg.r[0], IMM64);
}

#[test]
fn test_exec_imulh_r() {
    let instr = Instr {
        op: Opcode::IMULH_R,
        dst: r_reg(0),
        src: r_reg(1),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_imulh_r,
    };
    let mut vm = new_test_vm();
    vm.reg.r[0] = 0xBC550E96BA88A72B;
    vm.reg.r[1] = 0xF5391FA9F18D6273;

    instr.execute(&mut vm);
    assert_eq!(vm.reg.r[0], 0xB4676D31D2B34883);
}

#[test]
fn test_exec_ismulh_r() {
    let instr = Instr {
        op: Opcode::ISMULH_R,
        dst: r_reg(0),
        src: r_reg(1),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_ismulh_r,
    };
    let mut vm = new_test_vm();
    vm.reg.r[0] = 0xBC550E96BA88A72B;
    vm.reg.r[1] = 0xF5391FA9F18D6273;

    instr.execute(&mut vm);
    assert_eq!(vm.reg.r[0], 0x02D93EF1269D3EE5);
}

#[test]
fn test_exec_ineg_r() {
    let instr = Instr {
        op: Opcode::INEG_R,
        dst: r_reg(0),
        src: Store::NONE,
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_ineg_r,
    };
    let mut vm = new_test_vm();
    vm.reg.r[0] = 0xFFFFFFFFFFFFFFFF;

    instr.execute(&mut vm);

    assert_eq!(vm.reg.r[0], 1);
}

#[test]
fn test_exec_ineg_r_overflow() {
    let instr = Instr {
        op: Opcode::INEG_R,
        dst: r_reg(0),
        src: Store::NONE,
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_ineg_r,
    };
    let mut vm = new_test_vm();
    vm.reg.r[0] = 0x0;

    instr.execute(&mut vm);

    assert_eq!(vm.reg.r[0], 0);
}

#[test]
fn test_exec_ixor_r() {
    let instr = Instr {
        op: Opcode::IXOR_R,
        dst: r_reg(0),
        src: r_reg(1),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_ixor_r,
    };
    let mut vm = new_test_vm();
    vm.reg.r[0] = 0x8888888888888888;
    vm.reg.r[1] = 0xAAAAAAAAAAAAAAAA;

    instr.execute(&mut vm);

    assert_eq!(vm.reg.r[0], 0x2222222222222222);
}

#[test]
fn test_exec_ixor_r_with_immediate() {
    let instr = Instr {
        op: Opcode::IXOR_R,
        dst: r_reg(0),
        src: Store::NONE,
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_ixor_r,
    };
    let mut vm = new_test_vm();
    vm.reg.r[0] = 0xFFFFFFFFFFFFFFFF;

    instr.execute(&mut vm);

    assert_eq!(vm.reg.r[0], !IMM64);
}

#[test]
fn test_exec_iror_r() {
    let instr = Instr {
        op: Opcode::IROR_R,
        dst: r_reg(0),
        src: r_reg(1),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_iror_r,
    };
    let mut vm = new_test_vm();
    vm.reg.r[0] = 953360005391419562;
    vm.reg.r[1] = 4569451684712230561;

    instr.execute(&mut vm);

    assert_eq!(vm.reg.r[0], 0xD835C455069D81EF);
}

#[test]
#[allow(overflowing_literals)]
fn test_exec_iror_r_with_immediate() {
    let instr = Instr {
        op: Opcode::IROR_R,
        dst: r_reg(0),
        src: Store::NONE,
        imm: Some(4569451684712230561 as i32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_iror_r,
    };
    let mut vm = new_test_vm();
    vm.reg.r[0] = 953360005391419562;

    instr.execute(&mut vm);

    assert_eq!(vm.reg.r[0], 0xD835C455069D81EF);
}

#[test]
fn test_exec_irol_r() {
    let instr = Instr {
        op: Opcode::IROL_R,
        dst: r_reg(0),
        src: r_reg(1),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_irol_r,
    };
    let mut vm = new_test_vm();
    vm.reg.r[0] = 953360005391419562;
    vm.reg.r[1] = 4569451684712230561;

    instr.execute(&mut vm);

    assert_eq!(vm.reg.r[0], 6978065200552740799);
}

#[test]
#[allow(overflowing_literals)]
fn test_exec_irol_r_with_immediate() {
    let instr = Instr {
        op: Opcode::IROL_R,
        dst: r_reg(0),
        src: Store::NONE,
        imm: Some(4569451684712230561 as i32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_irol_r,
    };
    let mut vm = new_test_vm();
    vm.reg.r[0] = 953360005391419562;

    instr.execute(&mut vm);

    assert_eq!(vm.reg.r[0], 6978065200552740799);
}

#[test]
fn test_exec_iswap_r() {
    let instr = Instr {
        op: Opcode::ISWAP_R,
        dst: r_reg(0),
        src: r_reg(1),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_iswap_r,
    };
    let mut vm = new_test_vm();
    vm.reg.r[0] = 953360005391419562;
    vm.reg.r[1] = 4569451684712230561;

    instr.execute(&mut vm);
    assert_eq!(vm.reg.r[0], 4569451684712230561);
    assert_eq!(vm.reg.r[1], 953360005391419562);
}

#[test]
fn test_exec_fswap_r_from_f_reg() {
    let instr = Instr {
        op: Opcode::FSWAP_R,
        dst: f_reg(0),
        src: Store::NONE,
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_fswap_r,
    };
    let mut vm = new_test_vm();
    vm.reg.f[0] = m128d::from_u64(953360005391419562, 4569451684712230561);

    instr.execute(&mut vm);

    assert_eq!(
        vm.reg.f[0],
        m128d::from_u64(4569451684712230561, 953360005391419562)
    );
}

#[test]
fn test_exec_fswap_r_from_e_reg() {
    let instr = Instr {
        op: Opcode::FSWAP_R,
        dst: e_reg(3),
        src: Store::NONE,
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_fswap_r,
    };
    let mut vm = new_test_vm();
    vm.reg.e[3] = m128d::from_u64(953360005391419562, 4569451684712230561);

    instr.execute(&mut vm);

    assert_eq!(
        vm.reg.e[3],
        m128d::from_u64(4569451684712230561, 953360005391419562)
    );
}

#[test]
fn test_exec_fadd_r_round_to_nearest() {
    let instr = Instr {
        op: Opcode::FADD_R,
        dst: f_reg(0),
        src: a_reg(1),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_fadd_r,
    };
    let mut vm = new_test_vm();
    vm.set_rounding_mode(ROUND_TO_NEAREST);

    vm.reg.f[0] = m128d::from_u64(0x3ffd2c97cc4ef015, 0xc1ce30b3c4223576);
    vm.reg.a[1] = m128d::from_u64(0x402a26a86a60c8fb, 0x40b8f684057a59e1);

    instr.execute(&mut vm);

    assert_eq!(
        vm.reg.f[0],
        m128d::from_u64(0x402dcc3b63eaa6fe, 0xc1ce30a748e032b9)
    )
}

#[test]
fn test_exec_fadd_r_round_down() {
    let instr = Instr {
        op: Opcode::FADD_R,
        dst: f_reg(0),
        src: a_reg(1),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_fadd_r,
    };
    let mut vm = new_test_vm();
    vm.set_rounding_mode(ROUND_DOWN);

    vm.reg.f[0] = m128d::from_u64(0x3ffd2c97cc4ef015, 0xc1ce30b3c4223576);
    vm.reg.a[1] = m128d::from_u64(0x402a26a86a60c8fb, 0x40b8f684057a59e1);

    instr.execute(&mut vm);

    assert_eq!(
        vm.reg.f[0],
        m128d::from_u64(0x402dcc3b63eaa6fd, 0xc1ce30a748e032b9)
    )
}

#[test]
fn test_exec_fadd_r_round_up() {
    let instr = Instr {
        op: Opcode::FADD_R,
        dst: f_reg(0),
        src: a_reg(1),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_fadd_r,
    };
    let mut vm = new_test_vm();
    vm.set_rounding_mode(ROUND_UP);

    vm.reg.f[0] = m128d::from_u64(0x3ffd2c97cc4ef015, 0xc1ce30b3c4223576);
    vm.reg.a[1] = m128d::from_u64(0x402a26a86a60c8fb, 0x40b8f684057a59e1);

    instr.execute(&mut vm);

    assert_eq!(
        vm.reg.f[0],
        m128d::from_u64(0x402dcc3b63eaa6fe, 0xc1ce30a748e032b8)
    );
}

#[test]
fn test_exec_fadd_r_round_to_zero() {
    let instr = Instr {
        op: Opcode::FADD_R,
        dst: f_reg(0),
        src: a_reg(1),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_fadd_r,
    };
    let mut vm = new_test_vm();
    vm.set_rounding_mode(ROUND_TO_ZERO);

    vm.reg.f[0] = m128d::from_u64(0x3ffd2c97cc4ef015, 0xc1ce30b3c4223576);
    vm.reg.a[1] = m128d::from_u64(0x402a26a86a60c8fb, 0x40b8f684057a59e1);

    instr.execute(&mut vm);

    assert_eq!(
        vm.reg.f[0],
        m128d::from_u64(0x402dcc3b63eaa6fd, 0xc1ce30a748e032b8)
    )
}

#[test]
fn test_exec_fsub_r_round_to_nearest() {
    let instr = Instr {
        op: Opcode::FSUB_R,
        dst: f_reg(0),
        src: a_reg(1),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_fsub_r,
    };
    let mut vm = new_test_vm();
    vm.set_rounding_mode(ROUND_TO_NEAREST);

    vm.reg.f[0] = m128d::from_u64(0x3ffd2c97cc4ef015, 0xc1ce30b3c4223576);
    vm.reg.a[1] = m128d::from_u64(0x402a26a86a60c8fb, 0x40b8f684057a59e1);

    instr.execute(&mut vm);

    assert_eq!(
        vm.reg.f[0],
        m128d::from_u64(0xc026811570d6eaf8, 0xc1ce30c03f643833)
    )
}

#[test]
fn test_exec_fsub_r_round_down() {
    let instr = Instr {
        op: Opcode::FSUB_R,
        dst: f_reg(0),
        src: a_reg(1),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_fsub_r,
    };
    let mut vm = new_test_vm();
    vm.set_rounding_mode(ROUND_DOWN);

    vm.reg.f[0] = m128d::from_u64(0x3ffd2c97cc4ef015, 0xc1ce30b3c4223576);
    vm.reg.a[1] = m128d::from_u64(0x402a26a86a60c8fb, 0x40b8f684057a59e1);

    instr.execute(&mut vm);

    assert_eq!(
        vm.reg.f[0],
        m128d::from_u64(0xc026811570d6eaf9, 0xc1ce30c03f643834)
    )
}

#[test]
fn test_exec_fsub_r_round_up() {
    let instr = Instr {
        op: Opcode::FSUB_R,
        dst: f_reg(0),
        src: a_reg(1),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_fsub_r,
    };
    let mut vm = new_test_vm();
    vm.set_rounding_mode(ROUND_UP);

    vm.reg.f[0] = m128d::from_u64(0x3ffd2c97cc4ef015, 0xc1ce30b3c4223576);
    vm.reg.a[1] = m128d::from_u64(0x402a26a86a60c8fb, 0x40b8f684057a59e1);

    instr.execute(&mut vm);

    assert_eq!(
        vm.reg.f[0],
        m128d::from_u64(0xc026811570d6eaf8, 0xc1ce30c03f643833)
    )
}

#[test]
fn test_exec_fsub_r_round_to_zero() {
    let instr = Instr {
        op: Opcode::FSUB_R,
        dst: f_reg(0),
        src: a_reg(1),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_fsub_r,
    };
    let mut vm = new_test_vm();
    vm.set_rounding_mode(ROUND_TO_ZERO);

    vm.reg.f[0] = m128d::from_u64(0x3ffd2c97cc4ef015, 0xc1ce30b3c4223576);
    vm.reg.a[1] = m128d::from_u64(0x402a26a86a60c8fb, 0x40b8f684057a59e1);

    instr.execute(&mut vm);

    assert_eq!(
        vm.reg.f[0],
        m128d::from_u64(0xc026811570d6eaf8, 0xc1ce30c03f643833)
    )
}

#[test]
fn test_exec_fscal_r() {
    let instr = Instr {
        op: Opcode::FSCAL_R,
        dst: f_reg(0),
        src: Store::L1(Box::new(Store::R(1))),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_fscal_r,
    };
    let mut vm = new_test_vm();
    vm.reg.f[0] = m128d::from_u64(0x41dbc35cef248783, 0x40fdfdabb6173d07);
    instr.execute(&mut vm);

    assert_eq!(
        vm.reg.f[0],
        m128d::from_u64(0xc12bc35cef248783, 0xc00dfdabb6173d07)
    );
}

#[test]
fn test_exec_fmul_r_round_to_nearest() {
    let instr = Instr {
        op: Opcode::FMUL_R,
        dst: e_reg(0),
        src: a_reg(1),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_fmul_r,
    };
    let mut vm = new_test_vm();
    vm.set_rounding_mode(ROUND_TO_NEAREST);

    vm.reg.e[0] = m128d::from_u64(0x41dbc35cef248783, 0x40fdfdabb6173d07);
    vm.reg.a[1] = m128d::from_u64(0x40eba861aa31c7c0, 0x41c4561212ae2d50);

    instr.execute(&mut vm);

    assert_eq!(
        vm.reg.e[0],
        m128d::from_u64(0x42d7feeccd89152f, 0x42d30f35ff7a6969)
    )
}

#[test]
fn test_exec_fmul_r_round_round_down() {
    let instr = Instr {
        op: Opcode::FMUL_R,
        dst: e_reg(0),
        src: a_reg(1),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_fmul_r,
    };
    let mut vm = new_test_vm();
    vm.set_rounding_mode(ROUND_DOWN);

    vm.reg.e[0] = m128d::from_u64(0x41dbc35cef248783, 0x40fdfdabb6173d07);
    vm.reg.a[1] = m128d::from_u64(0x40eba861aa31c7c0, 0x41c4561212ae2d50);

    instr.execute(&mut vm);

    assert_eq!(
        vm.reg.e[0],
        m128d::from_u64(0x42d7feeccd89152e, 0x42d30f35ff7a6969)
    )
}

#[test]
fn test_exec_fmul_r_round_up() {
    let instr = Instr {
        op: Opcode::FMUL_R,
        dst: e_reg(0),
        src: a_reg(1),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_fmul_r,
    };
    let mut vm = new_test_vm();
    vm.set_rounding_mode(ROUND_UP);

    vm.reg.e[0] = m128d::from_u64(0x41dbc35cef248783, 0x40fdfdabb6173d07);
    vm.reg.a[1] = m128d::from_u64(0x40eba861aa31c7c0, 0x41c4561212ae2d50);

    instr.execute(&mut vm);

    assert_eq!(
        vm.reg.e[0],
        m128d::from_u64(0x42d7feeccd89152f, 0x42d30f35ff7a696a)
    )
}

#[test]
fn test_exec_fmul_r_round_to_zero() {
    let instr = Instr {
        op: Opcode::FMUL_R,
        dst: e_reg(0),
        src: a_reg(1),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_fmul_r,
    };
    let mut vm = new_test_vm();
    vm.set_rounding_mode(ROUND_TO_ZERO);

    vm.reg.e[0] = m128d::from_u64(0x41dbc35cef248783, 0x40fdfdabb6173d07);
    vm.reg.a[1] = m128d::from_u64(0x40eba861aa31c7c0, 0x41c4561212ae2d50);

    instr.execute(&mut vm);

    assert_eq!(
        vm.reg.e[0],
        m128d::from_u64(0x42d7feeccd89152e, 0x42d30f35ff7a6969)
    )
}

#[test]
fn test_exec_fsqrt_r_round_to_nearest() {
    let instr = Instr {
        op: Opcode::FSQRT_R,
        dst: e_reg(0),
        src: Store::NONE,
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_fsqrt_r,
    };
    let mut vm = new_test_vm();
    vm.set_rounding_mode(ROUND_TO_NEAREST);

    vm.reg.e[0] = m128d::from_u64(0x41b6b21c11affea7, 0x40526a7e778d9824);

    instr.execute(&mut vm);

    assert_eq!(
        vm.reg.e[0],
        m128d::from_u64(0x40d30e573fa3ba8d, 0x40212a610b301fe8)
    );
}

#[test]
fn test_exec_fsqrt_r_round_up() {
    let instr = Instr {
        op: Opcode::FSQRT_R,
        dst: e_reg(0),
        src: Store::NONE,
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_fsqrt_r,
    };
    let mut vm = new_test_vm();
    vm.set_rounding_mode(ROUND_UP);

    vm.reg.e[0] = m128d::from_u64(0x41b6b21c11affea7, 0x40526a7e778d9824);

    instr.execute(&mut vm);

    assert_eq!(
        vm.reg.e[0],
        m128d::from_u64(0x40d30e573fa3ba8d, 0x40212a610b301fe9)
    );
}

#[test]
fn test_exec_fsqrt_r_round_down() {
    let instr = Instr {
        op: Opcode::FSQRT_R,
        dst: e_reg(0),
        src: Store::NONE,
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_fsqrt_r,
    };
    let mut vm = new_test_vm();
    vm.set_rounding_mode(ROUND_DOWN);

    vm.reg.e[0] = m128d::from_u64(0x41b6b21c11affea7, 0x40526a7e778d9824);

    instr.execute(&mut vm);

    assert_eq!(
        vm.reg.e[0],
        m128d::from_u64(0x40d30e573fa3ba8c, 0x40212a610b301fe8)
    );
}

#[test]
fn test_exec_fsqrt_r_round_to_zero() {
    let instr = Instr {
        op: Opcode::FSQRT_R,
        dst: e_reg(0),
        src: Store::NONE,
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_fsqrt_r,
    };
    let mut vm = new_test_vm();
    vm.set_rounding_mode(ROUND_TO_ZERO);

    vm.reg.e[0] = m128d::from_u64(0x41b6b21c11affea7, 0x40526a7e778d9824);

    instr.execute(&mut vm);

    assert_eq!(
        vm.reg.e[0],
        m128d::from_u64(0x40d30e573fa3ba8c, 0x40212a610b301fe8)
    );
}

#[test]
#[allow(overflowing_literals)]
fn test_exec_fadd_m() {
    let instr = Instr {
        op: Opcode::FADD_M,
        dst: f_reg(0),
        src: Store::L1(Box::new(Store::R(1))),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_fadd_m,
    };
    let mut vm = new_test_vm();
    vm.scratchpad[0] = 0x1234567890abcdef;
    vm.reg.r[1] = 0xFFFFFFFFFFFFE930;
    vm.reg.f[0] = m128d::zero();
    instr.execute(&mut vm);

    assert_eq!(
        vm.reg.f[0],
        m128d::from_u64(0x41b2345678000000, 0xc1dbd50c84400000)
    );
}

#[test]
fn test_exec_fsub_m() {
    let instr = Instr {
        op: Opcode::FSUB_M,
        dst: f_reg(0),
        src: Store::L1(Box::new(r_reg(1))),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_fsub_m,
    };
    let mut vm = new_test_vm();
    vm.reg.r[1] = 0xFFFFFFFFFFFFE930;
    vm.reg.f[0] = m128d::from_u64(0x3ffd2c97cc4ef015, 0xc1ce30b3c4223576);
    vm.scratchpad[0] = 0x0203;
    instr.execute(&mut vm);

    assert_eq!(
        vm.reg.f[0],
        m128d::from_u64(0x3ffd2c97cc4ef015, 0xc1ce30b4c5a23576)
    );
}

#[test]
fn test_exec_cfround() {
    let instr = Instr {
        op: Opcode::CFROUND,
        dst: Store::NONE,
        src: r_reg(0),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_cfround,
    };
    let mut vm = new_test_vm();
    vm.reg.r[0] = 0xFFFFFFFFFFFC6800;

    assert_eq!(vm.get_rounding_mode(), ROUND_TO_NEAREST); //new vm starts with default rounding mode

    instr.execute(&mut vm);

    assert_eq!(vm.get_rounding_mode(), ROUND_TO_ZERO);
}

#[test]
#[allow(overflowing_literals)]
fn test_exec_cbranch_taken() {
    let instr = Instr {
        op: Opcode::CBRANCH,
        dst: r_reg(0),
        src: Store::NONE,
        imm: Some(0xFFFFFFFFC0CB9AD2),
        unsigned_imm: false,
        mode: Mode::Cond(3),
        target: Some(100),
        effect: Vm::exec_cbranch,
    };
    let mut vm = new_test_vm();
    vm.pc = 200;
    vm.reg.r[0] = 0xFFFFFFFFFFFC6800;

    instr.execute(&mut vm);

    assert_eq!(vm.pc, 100)
}

#[test]
#[allow(overflowing_literals)]
fn test_exec_cbranch_not_taken() {
    let instr = Instr {
        op: Opcode::CBRANCH,
        dst: r_reg(0),
        src: Store::NONE,
        imm: Some(0xFFFFFFFFC0CB9AD2),
        unsigned_imm: false,
        mode: Mode::Cond(3),
        target: None,
        effect: Vm::exec_cbranch,
    };
    let mut vm = new_test_vm();
    vm.pc = 200;
    vm.reg.r[0] = 0;

    instr.execute(&mut vm);

    assert_eq!(vm.pc, 200)
}

#[test]
#[allow(overflowing_literals)]
fn test_exec_istore_l1() {
    let instr = Instr {
        op: Opcode::ISTORE,
        dst: Store::L1(Box::new(r_reg(0))),
        src: r_reg(1),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_istore,
    };
    let mut vm = new_test_vm();
    vm.reg.r[1] = 0xFFFFFFFFFFFC6800;
    vm.reg.r[0] = 0xFFFFFFFFC0C802D2;

    instr.execute(&mut vm);

    assert_eq!(vm.scratchpad[0x19A0 / 8], 0xFFFFFFFFFFFC6800);
}

#[test]
#[allow(overflowing_literals)]
fn test_exec_istore_l2() {
    let instr = Instr {
        op: Opcode::ISTORE,
        dst: Store::L2(Box::new(r_reg(0))),
        src: r_reg(1),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_istore,
    };
    let mut vm = new_test_vm();
    vm.reg.r[1] = 0xFFFFFFFFFFFC6800;
    vm.reg.r[0] = 0xFFFFFFFFC0C802D2;

    instr.execute(&mut vm);

    assert_eq!(vm.scratchpad[0x399A0 / 8], 0xFFFFFFFFFFFC6800);
}

#[test]
#[allow(overflowing_literals)]
fn test_exec_istore_l3() {
    let instr = Instr {
        op: Opcode::ISTORE,
        dst: Store::L3(Box::new(r_reg(0))),
        src: r_reg(1),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_istore,
    };
    let mut vm = new_test_vm();
    vm.reg.r[1] = 0xFFFFFFFFFFFC6800;
    vm.reg.r[0] = 0xFFFFFFFFC0C802D2;

    instr.execute(&mut vm);

    assert_eq!(vm.scratchpad[0x1399A0 / 8], 0xFFFFFFFFFFFC6800);
}

#[test]
fn test_exec_iadd_m_l1() {
    let instr = Instr {
        op: Opcode::IADD_M,
        dst: r_reg(0),
        src: Store::L1(Box::new(r_reg(1))),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_iadd_m,
    };
    let mut vm = new_test_vm();
    vm.reg.r[1] = 0xFFFFFFFFFFFFE930;
    vm.reg.r[0] = 0x666;
    vm.scratchpad[0] = 0x0203;
    instr.execute(&mut vm);

    assert_eq!(vm.reg.r[0], 0x869);
}

#[test]
fn test_exec_iadd_m_l2() {
    let instr = Instr {
        op: Opcode::IADD_M,
        dst: r_reg(0),
        src: Store::L2(Box::new(r_reg(1))),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_iadd_m,
    };
    let mut vm = new_test_vm();
    vm.reg.r[1] = 0xFFFFFFFFFFFFE930;
    vm.reg.r[0] = 0x666;
    vm.scratchpad[0x38000 / 8] = 0x0203;
    instr.execute(&mut vm);

    assert_eq!(vm.reg.r[0], 0x869);
}

#[test]
fn test_exec_iadd_m_l3() {
    let instr = Instr {
        op: Opcode::IADD_M,
        dst: r_reg(0),
        src: Store::L3(Box::new(r_reg(1))),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_iadd_m,
    };
    let mut vm = new_test_vm();
    vm.reg.r[1] = 0xFFFFFFFFFFFFE930;
    vm.reg.r[0] = 0x666;
    vm.scratchpad[0xb96d0 / 8] = 0x0203;
    instr.execute(&mut vm);

    assert_eq!(vm.reg.r[0], 0x869);
}

#[test]
fn test_exec_isub_m_l1() {
    let instr = Instr {
        op: Opcode::ISUB_M,
        dst: r_reg(0),
        src: Store::L1(Box::new(r_reg(1))),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_isub_m,
    };
    let mut vm = new_test_vm();
    vm.reg.r[1] = 0xFFFFFFFFFFFFE930;
    vm.reg.r[0] = 0x666;
    vm.scratchpad[0] = 0x0203;
    instr.execute(&mut vm);

    assert_eq!(vm.reg.r[0], 0x666 - 0x203);
}

#[test]
fn test_exec_isub_m_l2() {
    let instr = Instr {
        op: Opcode::ISUB_M,
        dst: r_reg(0),
        src: Store::L2(Box::new(r_reg(1))),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_isub_m,
    };
    let mut vm = new_test_vm();
    vm.reg.r[1] = 0xFFFFFFFFFFFFE930;
    vm.reg.r[0] = 0x666;
    vm.scratchpad[0x38000 / 8] = 0x0203;
    instr.execute(&mut vm);

    assert_eq!(vm.reg.r[0], 0x666 - 0x0203);
}

#[test]
fn test_exec_isub_m_l3() {
    let instr = Instr {
        op: Opcode::ISUB_M,
        dst: r_reg(0),
        src: Store::L3(Box::new(r_reg(1))),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_isub_m,
    };
    let mut vm = new_test_vm();
    vm.reg.r[1] = 0xFFFFFFFFFFFFE930;
    vm.reg.r[0] = 0x666;
    vm.scratchpad[0xb96d0 / 8] = 0x0203;
    instr.execute(&mut vm);

    assert_eq!(vm.reg.r[0], 0x666 - 0x0203);
}

#[test]
fn test_exec_imul_m_l1() {
    let instr = Instr {
        op: Opcode::IMUL_M,
        dst: r_reg(0),
        src: Store::L1(Box::new(r_reg(1))),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_imul_m,
    };
    let mut vm = new_test_vm();
    vm.reg.r[1] = 0xFFFFFFFFFFFFE930;
    vm.reg.r[0] = 0x666;
    vm.scratchpad[0] = 0x0203;
    instr.execute(&mut vm);

    assert_eq!(vm.reg.r[0], 0x666 * 0x203);
}

#[test]
fn test_exec_imul_m_l2() {
    let instr = Instr {
        op: Opcode::IMUL_M,
        dst: r_reg(0),
        src: Store::L2(Box::new(r_reg(1))),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_imul_m,
    };
    let mut vm = new_test_vm();
    vm.reg.r[1] = 0xFFFFFFFFFFFFE930;
    vm.reg.r[0] = 0x666;
    vm.scratchpad[0x38000 / 8] = 0x0203;
    instr.execute(&mut vm);

    assert_eq!(vm.reg.r[0], 0x666 * 0x0203);
}

#[test]
fn test_exec_imul_m_l3() {
    let instr = Instr {
        op: Opcode::IMUL_M,
        dst: r_reg(0),
        src: Store::L3(Box::new(r_reg(1))),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_imul_m,
    };
    let mut vm = new_test_vm();
    vm.reg.r[1] = 0xFFFFFFFFFFFFE930;
    vm.reg.r[0] = 0x666;
    vm.scratchpad[0xb96d0 / 8] = 0x0203;
    instr.execute(&mut vm);

    assert_eq!(vm.reg.r[0], 0x666 * 0x0203);
}

#[test]
fn test_exec_imulh_m_l1() {
    let instr = Instr {
        op: Opcode::IMULH_M,
        dst: r_reg(0),
        src: Store::L1(Box::new(r_reg(1))),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_imulh_m,
    };
    let mut vm = new_test_vm();
    vm.reg.r[0] = 0xBC550E96BA88A72B;
    vm.reg.r[1] = 0xFFFFFFFFFFFFE930;
    vm.scratchpad[0] = 0xF5391FA9F18D6273;

    instr.execute(&mut vm);

    assert_eq!(vm.reg.r[0], 0xB4676D31D2B34883);
}

#[test]
fn test_exec_imulh_m_l2() {
    let instr = Instr {
        op: Opcode::IMULH_M,
        dst: r_reg(0),
        src: Store::L2(Box::new(r_reg(1))),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_imulh_m,
    };
    let mut vm = new_test_vm();
    vm.reg.r[0] = 0xBC550E96BA88A72B;
    vm.reg.r[1] = 0xFFFFFFFFFFFFE930;
    vm.scratchpad[0x38000 / 8] = 0xF5391FA9F18D6273;

    instr.execute(&mut vm);

    assert_eq!(vm.reg.r[0], 0xB4676D31D2B34883);
}

#[test]
fn test_exec_imulh_m_l3() {
    let instr = Instr {
        op: Opcode::IMULH_M,
        dst: r_reg(0),
        src: Store::L3(Box::new(r_reg(1))),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_imulh_m,
    };
    let mut vm = new_test_vm();
    vm.reg.r[0] = 0xBC550E96BA88A72B;
    vm.reg.r[1] = 0xFFFFFFFFFFFFE930;
    vm.scratchpad[0xb96d0 / 8] = 0xF5391FA9F18D6273;

    instr.execute(&mut vm);

    assert_eq!(vm.reg.r[0], 0xB4676D31D2B34883);
}

#[test]
fn test_exec_ismulh_m_l1() {
    let instr = Instr {
        op: Opcode::ISMULH_M,
        dst: r_reg(0),
        src: Store::L1(Box::new(r_reg(1))),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_ismulh_m,
    };
    let mut vm = new_test_vm();
    vm.reg.r[0] = 0xBC550E96BA88A72B;
    vm.reg.r[1] = 0xFFFFFFFFFFFFE930;
    vm.scratchpad[0] = 0xF5391FA9F18D6273;

    instr.execute(&mut vm);

    assert_eq!(vm.reg.r[0], 0x02D93EF1269D3EE5);
}

#[test]
fn test_exec_ismulh_m_l2() {
    let instr = Instr {
        op: Opcode::ISMULH_M,
        dst: r_reg(0),
        src: Store::L2(Box::new(r_reg(1))),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_ismulh_m,
    };
    let mut vm = new_test_vm();
    vm.reg.r[0] = 0xBC550E96BA88A72B;
    vm.reg.r[1] = 0xFFFFFFFFFFFFE930;
    vm.scratchpad[0x38000 / 8] = 0xF5391FA9F18D6273;

    instr.execute(&mut vm);

    assert_eq!(vm.reg.r[0], 0x02D93EF1269D3EE5);
}

#[test]
fn test_exec_ismulh_m_l3() {
    let instr = Instr {
        op: Opcode::ISMULH_M,
        dst: r_reg(0),
        src: Store::L3(Box::new(r_reg(1))),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_ismulh_m,
    };
    let mut vm = new_test_vm();
    vm.reg.r[0] = 0xBC550E96BA88A72B;
    vm.reg.r[1] = 0xFFFFFFFFFFFFE930;
    vm.scratchpad[0xb96d0 / 8] = 0xF5391FA9F18D6273;

    instr.execute(&mut vm);

    assert_eq!(vm.reg.r[0], 0x02D93EF1269D3EE5);
}

#[test]
fn test_exec_imul_rcp_non_zero_imm_from_reg() {
    let instr = Instr {
        op: Opcode::IMUL_RCP,
        dst: r_reg(0),
        src: Store::NONE,
        imm: Some(IMM32),
        unsigned_imm: true,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_imul_rcp,
    };
    let mut vm = new_test_vm();
    vm.reg.r[0] = 666;

    instr.execute(&mut vm);

    assert_eq!(vm.reg.r[0], 0x2B2462DE8506B218);
}

#[test]
fn test_exec_imul_rcp_zero_imm() {
    let instr = Instr {
        op: Opcode::IMUL_RCP,
        dst: r_reg(0),
        src: r_reg(1),
        imm: Some(0),
        unsigned_imm: true,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_imul_rcp,
    };
    let mut vm = new_test_vm();
    vm.reg.r[0] = 0x666;

    instr.execute(&mut vm);

    assert_eq!(vm.reg.r[0], 0x666);
}

#[test]
fn test_exec_ixor_m_l1() {
    let instr = Instr {
        op: Opcode::IXOR_M,
        dst: r_reg(0),
        src: Store::L1(Box::new(r_reg(1))),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_ixor_m,
    };
    let mut vm = new_test_vm();
    vm.reg.r[1] = 0xFFFFFFFFFFFFE930;
    vm.reg.r[0] = 0x666;
    vm.scratchpad[0] = 0x0203;
    instr.execute(&mut vm);

    assert_eq!(vm.reg.r[0], 0x666 ^ 0x203);
}

#[test]
fn test_exec_ixor_m_l2() {
    let instr = Instr {
        op: Opcode::IXOR_M,
        dst: r_reg(0),
        src: Store::L2(Box::new(r_reg(1))),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_ixor_m,
    };
    let mut vm = new_test_vm();
    vm.reg.r[1] = 0xFFFFFFFFFFFFE930;
    vm.reg.r[0] = 0x666;
    vm.scratchpad[0x38000 / 8] = 0x0203;
    instr.execute(&mut vm);

    assert_eq!(vm.reg.r[0], 0x666 ^ 0x203);
}

#[test]
fn test_exec_ixor_m_l3() {
    let instr = Instr {
        op: Opcode::IXOR_M,
        dst: r_reg(0),
        src: Store::L3(Box::new(r_reg(1))),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_ixor_m,
    };
    let mut vm = new_test_vm();
    vm.reg.r[1] = 0xFFFFFFFFFFFFE930;
    vm.reg.r[0] = 0x666;
    vm.scratchpad[0xb96d0 / 8] = 0x0203;
    instr.execute(&mut vm);

    assert_eq!(vm.reg.r[0], 0x666 ^ 0x203);
}

#[test]
fn test_exec_fdiv_m_round_to_nearest() {
    let instr = Instr {
        op: Opcode::FDIV_M,
        dst: e_reg(0),
        src: Store::L1(Box::new(r_reg(1))),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_fdiv_m,
    };
    let mut vm = new_test_vm();
    vm.reg.r[1] = 0xFFFFFFFFFFFFE930;
    vm.reg.e[0] = m128d::from_u64(0x41937f76fede16ee, 0x411b414296ce93b6);
    vm.set_rounding_mode(ROUND_TO_NEAREST);
    vm.config.e_mask[0] = 0x3a0000000005d11a;
    vm.config.e_mask[1] = 0x39000000001ba31e;
    vm.scratchpad[0] = 0x8b2460d9d350a1b6;

    instr.execute(&mut vm);

    assert_eq!(
        vm.reg.e[0],
        m128d::from_u64(0x47a55b63664a4732, 0x464384946369b2e7)
    );
}

#[test]
fn test_exec_fdiv_m_round_down_and_to_zero() {
    let instr = Instr {
        op: Opcode::FDIV_M,
        dst: e_reg(0),
        src: Store::L1(Box::new(r_reg(1))),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_fdiv_m,
    };
    let mut vm = new_test_vm();
    vm.reg.r[1] = 0xFFFFFFFFFFFFE930;
    vm.reg.e[0] = m128d::from_u64(0x41937f76fede16ee, 0x411b414296ce93b6);
    vm.set_rounding_mode(ROUND_TO_ZERO);
    vm.config.e_mask[0] = 0x3a0000000005d11a;
    vm.config.e_mask[1] = 0x39000000001ba31e;
    vm.scratchpad[0] = 0x8b2460d9d350a1b6;

    instr.execute(&mut vm);

    assert_eq!(
        vm.reg.e[0],
        m128d::from_u64(0x47a55b63664a4732, 0x464384946369b2e6)
    );
}

#[test]
fn test_exec_fdiv_m_round_to_zero() {
    let instr = Instr {
        op: Opcode::FDIV_M,
        dst: e_reg(0),
        src: Store::L1(Box::new(r_reg(1))),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_fdiv_m,
    };
    let mut vm = new_test_vm();
    vm.reg.r[1] = 0xFFFFFFFFFFFFE930;
    vm.reg.e[0] = m128d::from_u64(0x41937f76fede16ee, 0x411b414296ce93b6);
    vm.set_rounding_mode(ROUND_DOWN);
    vm.config.e_mask[0] = 0x3a0000000005d11a;
    vm.config.e_mask[1] = 0x39000000001ba31e;
    vm.scratchpad[0] = 0x8b2460d9d350a1b6;

    instr.execute(&mut vm);

    assert_eq!(
        vm.reg.e[0],
        m128d::from_u64(0x47a55b63664a4732, 0x464384946369b2e6)
    );
}

#[test]
fn test_exec_fdiv_m_round_up() {
    let instr = Instr {
        op: Opcode::FDIV_M,
        dst: e_reg(0),
        src: Store::L1(Box::new(r_reg(1))),
        imm: Some(IMM32),
        unsigned_imm: false,
        mode: Mode::None,
        target: None,
        effect: Vm::exec_fdiv_m,
    };
    let mut vm = new_test_vm();
    vm.reg.r[1] = 0xFFFFFFFFFFFFE930;
    vm.reg.e[0] = m128d::from_u64(0x41937f76fede16ee, 0x411b414296ce93b6);
    vm.set_rounding_mode(ROUND_UP);
    vm.config.e_mask[0] = 0x3a0000000005d11a;
    vm.config.e_mask[1] = 0x39000000001ba31e;
    vm.scratchpad[0] = 0x8b2460d9d350a1b6;

    instr.execute(&mut vm);

    assert_eq!(
        vm.reg.e[0],
        m128d::from_u64(0x47a55b63664a4733, 0x464384946369b2e7)
    );
}

#[test]
fn test_randomx_reciprocal() {
    let result = randomx_reciprocal(0xc0cb96d2);
    assert_eq!(result, 0xa9f671ed1d69b73c);
}

//helper

fn new_test_vm() -> Vm {
    new_vm(Arc::new(VmMemory::no_memory()))
}
