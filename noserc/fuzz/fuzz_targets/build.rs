#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate noserc;
extern crate noser;

use noserc::{Build, SizableStatic, SizableDynamic};
use noser::{Literal, List, traits::Build};

#[derive(SizableStatic, Build)]
struct StaticStruct<'a> {
    lit_char: Literal<'a, char>,
    lit_bool: Literal<'a, bool>,
    lit_u8: Literal<'a, u8>,
    lit_i8: Literal<'a, i8>,
    lit_u16: Literal<'a, u16>,
}

#[derive(SizableDynamic, Build)]
struct DynamicStruct<'a> {
    list_f32: List<'a, Literal<'a, f32>>,
}

#[derive(SizableStatic, Build)]
struct StructNamed<'a> {
    named_field: Literal<'a, f32>
}

#[derive(SizableStatic, Build)]
struct StructUnnamed<'a>(Literal<'a, f32>);

#[derive(SizableStatic, Build)]
struct StructVoid;

#[derive(SizableDynamic, Build)]
enum DynamicEnum<'a> {
    Var1(Literal<'a, u64>),
    Var2(Literal<'a, u8>),
}

#[derive(SizableDynamic, Build)]
enum EnumSingle<'a> {
    Var(Literal<'a, u64>),
}

#[derive(SizableDynamic, Build)]
enum EnumTwo<'a> {
    Var1(Literal<'a, u64>),
    Var2,
}

#[derive(SizableDynamic, Build)]
enum EnumMultiple {
    Var1, Var2, Var3, Var4, Var5, Var6, Var7, Var8, Var9, Var10,
    Var11, Var12, Var13, Var14, Var15, Var16, Var17, Var18, Var19, Var20,
    Var21, Var22, Var23, Var24, Var25, Var26, Var27, Var28, Var29, Var30,
    Var31, Var32, Var33, Var34, Var35, Var36, Var37, Var38, Var39, Var40,
    Var41, Var42, Var43, Var44, Var45, Var46, Var47, Var48, Var49, Var50,
    Var51, Var52, Var53, Var54, Var55, Var56, Var57, Var58, Var59, Var60,
    Var61, Var62, Var63, Var64, Var65, Var66, Var67, Var68, Var69, Var70,
    Var71, Var72, Var73, Var74, Var75, Var76, Var77, Var78, Var79, Var80,
    Var81, Var82, Var83, Var84, Var85, Var86, Var87, Var88, Var89, Var90,
    Var91, Var92, Var93, Var94, Var95, Var96, Var97, Var98, Var99, Var100,
    Var101, Var102, Var103, Var104, Var105, Var106, Var107, Var108, Var109, Var110,
    Var111, Var112, Var113, Var114, Var115, Var116, Var117, Var118, Var119, Var120,
    Var121, Var122, Var123, Var124, Var125, Var126, Var127, Var128, Var129, Var130,
    Var131, Var132, Var133, Var134, Var135, Var136, Var137, Var138, Var139, Var140,
    Var141, Var142, Var143, Var144, Var145, Var146, Var147, Var148, Var149, Var150,
    Var151, Var152, Var153, Var154, Var155, Var156, Var157, Var158, Var159, Var160,
    Var161, Var162, Var163, Var164, Var165, Var166, Var167, Var168, Var169, Var170,
    Var171, Var172, Var173, Var174, Var175, Var176, Var177, Var178, Var179, Var180,
    Var181, Var182, Var183, Var184, Var185, Var186, Var187, Var188, Var189, Var190,
    Var191, Var192, Var193, Var194, Var195, Var196, Var197, Var198, Var199, Var200,
    Var201, Var202, Var203, Var204, Var205, Var206, Var207, Var208, Var209, Var210,
    Var211, Var212, Var213, Var214, Var215, Var216, Var217, Var218, Var219, Var220,
    Var221, Var222, Var223, Var224, Var225, Var226, Var227, Var228, Var229, Var230,
    Var231, Var232, Var233, Var234, Var235, Var236, Var237, Var238, Var239, Var240,
    Var241, Var242, Var243, Var244, Var245, Var246, Var247, Var248, Var249, Var250,
    Var251, Var252, Var253, Var254, Var255, Var256, Var257, Var258, Var259, Var260,
    Var261, Var262, Var263, Var264, Var265, Var266, Var267, Var268, Var269, Var270,
}

#[derive(SizableDynamic, Build)]
struct FuzzType<'a> {
    lit_char: Literal<'a, char>,
    lit_bool: Literal<'a, bool>,
    lit_u8: Literal<'a, u8>,
    lit_i8: Literal<'a, i8>,
    lit_u16: Literal<'a, u16>,
    lit_i16: Literal<'a, i16>,
    lit_u32: Literal<'a, u32>,
    lit_i32: Literal<'a, i32>,
    lit_u64: Literal<'a, u64>,
    lit_i64: Literal<'a, i64>,
    #[cfg(feature = "u128")]
    lit_u128: Literal<'a, u128>,
    #[cfg(feature = "i128")]
    lit_i128: Literal<'a, i128>,
    lit_f32: Literal<'a, f32>,
    lit_f64: Literal<'a, f64>,
    lit_slice: Literal<'a, &'a [u8]>,
    lit_str: Literal<'a, &'a str>,
    list_f32: List<'a, Literal<'a, f32>>,
    nested_list_f32: List<'a, List<'a, Literal<'a, f32>>>,
    static_struct: StaticStruct<'a>,
    dynamic_struct: DynamicStruct<'a>,
    dynamic_enum: DynamicEnum<'a>,
    enum_single_var: EnumSingle<'a>,
    enum_two_var: EnumTwo<'a>,
    enum_multiple_var: EnumMultiple,
    struct_named: StructNamed<'a>,
    struct_unnamed: StructUnnamed<'a>,
    struct_void: StructVoid,
    list_static_struct: List<'a, StaticStruct<'a>>,
    list_dynamic_struct: List<'a, DynamicStruct<'a>>,
    list_dynamic_enum: List<'a, DynamicEnum<'a>>,
    nested_list_static_struct: List<'a, List<'a, StaticStruct<'a>>>,
}

fuzz_target!(|data: &[u8]| {
    if let Ok(fuzz) = FuzzType::create_read_only(data) {
        fuzz.lit_bool.read();
        fuzz.lit_char.read();
        fuzz.lit_u8.read();
        fuzz.lit_i8.read();
        fuzz.lit_u16.read();
        fuzz.lit_i16.read();
        fuzz.lit_u32.read();
        fuzz.lit_i32.read();
        fuzz.lit_u64.read();
        fuzz.lit_i64.read();
        #[cfg(feature = "u128")]
        fuzz.lit_u128.read();
        #[cfg(feature = "i128")]
        fuzz.lit_i128.read();
        fuzz.lit_f32.read();
        fuzz.lit_f64.read();
        fuzz.lit_slice.read();
        fuzz.lit_str.read();

        // Struct
        fuzz.static_struct.lit_char.read();
        fuzz.static_struct.lit_bool.read();
        fuzz.static_struct.lit_u8.read();
        fuzz.static_struct.lit_i8.read();
        fuzz.static_struct.lit_u16.read();

        fuzz.struct_named.named_field.read();
        fuzz.struct_unnamed.0.read();

        let void = &fuzz.struct_void;

        // Enum
        match fuzz.dynamic_enum {
            DynamicEnum::Var1(ref lit_u64) => {
                lit_u64.read();
            },
            DynamicEnum::Var2(ref lit_u8) => {
                lit_u8.read();
            }
        };

        match fuzz.enum_single_var {
            EnumSingle::Var(ref lit_u64) => {
                lit_u64.read();
            }
        };

        match fuzz.enum_two_var {
            EnumTwo::Var1(ref lit_u64) => {
                lit_u64.read();
            },
            EnumTwo::Var2 => { }
        };

        let var = match fuzz.enum_multiple_var {
            EnumMultiple::Var1 => 1,
            EnumMultiple::Var2 => 2,
            EnumMultiple::Var256 => 256,
            EnumMultiple::Var260 => 260,
            _ => -1,
        };

        // Lists
        for i in 0..fuzz.list_f32.capacity() {
            fuzz.list_f32.borrow(i).read();
        }

        for i in 0..fuzz.list_static_struct.capacity() {
            let ss = fuzz.list_static_struct.borrow(i);

            ss.lit_char.read();
            ss.lit_bool.read();
            ss.lit_u8.read();
            ss.lit_i8.read();
            ss.lit_u16.read();
        }

        for i in 0..fuzz.list_dynamic_struct.capacity() {
            let ds = fuzz.list_dynamic_struct.borrow(i);

            for i in 0..ds.list_f32.capacity() {
                ds.list_f32.borrow(i).read();
            }
        }

        for i in 0..fuzz.list_dynamic_enum.capacity() {
            let de = fuzz.list_dynamic_enum.borrow(i);

            match *de {
                DynamicEnum::Var1(ref lit_u64) => {
                    lit_u64.read();
                },
                DynamicEnum::Var2(ref lit_u8) => {
                    lit_u8.read();
                }
            };
        }

        for i in 0..fuzz.dynamic_struct.list_f32.capacity() {
            fuzz.dynamic_struct.list_f32.borrow(i).read();
        }

        for i in 0..fuzz.nested_list_f32.capacity() {
            let inner = fuzz.nested_list_f32.borrow(i);

            for j in 0..inner.capacity() {
                inner.borrow(j).read();
            }
        }

        for i in 0..fuzz.nested_list_static_struct.capacity() {
            let inner = fuzz.nested_list_static_struct.borrow(i);

            for j in 0..inner.capacity() {
                let ss = inner.borrow(j);

                ss.lit_char.read();
                ss.lit_bool.read();
                ss.lit_u8.read();
                ss.lit_i8.read();
                ss.lit_u16.read();
            }
        }
    }
});
