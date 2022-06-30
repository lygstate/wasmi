//! Definitions for visualization of `wasmi` bytecode instruction.

use super::{
    DisplayExecProvider,
    DisplayExecProviderSlice,
    DisplayExecRegister,
    DisplayExecRegisterSlice,
    DisplayFuncIdx,
    DisplayFuncType,
    DisplayGlobal,
    DisplayTarget,
};
use crate::{
    engine::{
        bytecode::{ExecRegister, Offset},
        inner::EngineResources,
        ExecInstruction,
        ExecProvider,
        Instruction,
    },
    Instance,
    StoreContext,
};
use core::{fmt, fmt::Display};
use wasmi_core::TrapCode;

/// Wrapper to display an [`ExecInstruction`] in a human readable way.
#[derive(Debug)]
pub struct DisplayExecInstruction<'ctx, 'engine, T> {
    ctx: StoreContext<'ctx, T>,
    res: &'engine EngineResources,
    instance: Instance,
    instr: ExecInstruction,
}

impl<'ctx, 'engine, T> DisplayExecInstruction<'ctx, 'engine, T> {
    /// Creates a new [`DisplayExecInstruction`] wrapper.
    ///
    /// Used to write the [`ExecInstruction`] in a human readable form.
    pub fn new(
        ctx: StoreContext<'ctx, T>,
        res: &'engine EngineResources,
        instance: Instance,
        instr: &ExecInstruction,
    ) -> Self {
        Self {
            ctx,
            res,
            instance,
            instr: *instr,
        }
    }

    /// Writes human readable output for an unary `wasmi` instruction.
    fn write_unary(
        &self,
        f: &mut fmt::Formatter,
        name: &str,
        result: ExecRegister,
        input: ExecRegister,
    ) -> fmt::Result {
        writeln!(
            f,
            "{} <- {name} {}",
            DisplayExecRegister::from(result),
            DisplayExecRegister::from(input),
        )
    }

    /// Writes human readable output for a binary `wasmi` instruction.
    fn write_binary(
        &self,
        f: &mut fmt::Formatter,
        name: &str,
        result: ExecRegister,
        lhs: ExecRegister,
        rhs: ExecProvider,
    ) -> fmt::Result {
        writeln!(
            f,
            "{} <- {name} {} {}",
            DisplayExecRegister::from(result),
            DisplayExecRegister::from(lhs),
            DisplayExecProvider::new(self.res, rhs),
        )
    }

    /// Writes human readable output for a `wasmi` memory load instruction.
    fn write_load(
        &self,
        f: &mut fmt::Formatter,
        name: &str,
        result: ExecRegister,
        ptr: ExecRegister,
        offset: Offset,
    ) -> fmt::Result {
        writeln!(
            f,
            "{name} {} <- mem[{}+{}]",
            DisplayExecRegister::from(result),
            DisplayExecRegister::from(ptr),
            offset.into_inner(),
        )
    }

    /// Writes human readable output for a `wasmi` memory store instruction.
    fn write_store(
        &self,
        f: &mut fmt::Formatter,
        name: &str,
        ptr: ExecRegister,
        offset: Offset,
        value: ExecProvider,
    ) -> fmt::Result {
        writeln!(
            f,
            "{name} mem[{}+{}] <- {}",
            DisplayExecRegister::from(ptr),
            offset.into_inner(),
            DisplayExecProvider::new(self.res, value),
        )
    }
}

impl<T> Display for DisplayExecInstruction<'_, '_, T> {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = self.res;
        use Instruction as Instr;
        match self.instr {
            Instr::Br { target } => {
                writeln!(f, "br {}", DisplayTarget::from(target))
            }
            Instr::BrEqz { target, condition } => {
                writeln!(f, "br_eqz {} {}",
                    DisplayExecRegister::from(condition),
                    DisplayTarget::from(target),
                )
            }
            Instr::BrNez { target, condition } => {
                writeln!(f, "br_nez {} {}",
                    DisplayExecRegister::from(condition),
                    DisplayTarget::from(target),
                )
            }
            Instr::ReturnNez { results, condition } => {
                writeln!(
                    f,
                    "return_nez {} {}",
                    DisplayExecRegister::from(condition),
                    DisplayExecProviderSlice::new(res, results),
                )
            }
            Instr::BrTable { case: _, len_targets: _ } => todo!(),
            Instr::Trap { trap_code } => {
                let trap_name = match trap_code {
                    TrapCode::Unreachable => "unreachable",
                    TrapCode::MemoryAccessOutOfBounds => "memory_access_out_of_bounds",
                    TrapCode::TableAccessOutOfBounds => "table_access_out_of_bounds",
                    TrapCode::ElemUninitialized => "element_uninitialized",
                    TrapCode::DivisionByZero => "division_by_zero",
                    TrapCode::IntegerOverflow => "integer_overflow",
                    TrapCode::InvalidConversionToInt => "invalid_conversion_to_int",
                    TrapCode::StackOverflow => "stack_overflow",
                    TrapCode::UnexpectedSignature => "unexpected_signature",
                };
                writeln!(f, "trap -> {:?}", trap_name)
            }
            Instr::Return { results } => {
                writeln!(
                    f,
                    "return {}",
                    DisplayExecProviderSlice::new(res, results)
                )
            }
            Instr::Call {
                func_idx,
                results,
                params,
            } => {
                writeln!(f, "{} <- call {} {}",
                    DisplayExecRegisterSlice::from(results),
                    DisplayFuncIdx::from(func_idx),
                    DisplayExecProviderSlice::new(res, params),
                )
            }
            Instr::CallIndirect {
                func_type_idx,
                results,
                index,
                params,
            } => {
                let func_type = self.ctx.store.resolve_instance(self.instance)
                    .get_signature(func_type_idx.into_u32())
                    .unwrap_or_else(|| {
                        panic!(
                            "missing function type at index {} for call_indirect",
                            func_type_idx.into_u32(),
                        )
                    });
                let func_type = res.func_types.resolve_func_type(func_type);
                write!(
                    f,
                    "{} <- call_indirect table[{}] {}: {}",
                    DisplayExecRegisterSlice::from(results),
                    DisplayExecProvider::new(res, index),
                    DisplayExecProviderSlice::new(res, params),
                    DisplayFuncType::from(func_type),
                )
            }
            Instr::Copy { result, input } => {
                writeln!(f, "{} <- {}",
                    DisplayExecRegister::from(result),
                    DisplayExecProvider::new(res, input),
                )
            }
            Instr::Select {
                result,
                condition,
                if_true,
                if_false,
            } => {
                writeln!(f, "{} <- if {} then {} else {}",
                    DisplayExecRegister::from(result),
                    DisplayExecRegister::from(condition),
                    DisplayExecProvider::new(res, if_true),
                    DisplayExecProvider::new(res, if_false),
                )
            }
            Instr::GlobalGet { result, global } => {
                write!(
                    f,
                    "{} <- {}",
                    DisplayExecRegister::from(result),
                    DisplayGlobal::from(global),
                )
            }
            Instr::GlobalSet { global, value } => {
                write!(
                    f,
                    "{} <- {}",
                    DisplayGlobal::from(global),
                    DisplayExecProvider::new(res, value),
                )
            }
            Instr::I32Load { result, ptr, offset } => self.write_load(f, "i32.load", result, ptr, offset),
            Instr::I64Load { result, ptr, offset } => self.write_load(f, "i64.load", result, ptr, offset),
            Instr::F32Load { result, ptr, offset } => self.write_load(f, "f32.load", result, ptr, offset),
            Instr::F64Load { result, ptr, offset } => self.write_load(f, "f64.load", result, ptr, offset),
            Instr::I32Load8S { result, ptr, offset } => self.write_load(f, "i32.load8_s", result, ptr, offset),
            Instr::I32Load8U { result, ptr, offset } => self.write_load(f, "i32.load8_u", result, ptr, offset),
            Instr::I32Load16S { result, ptr, offset } => self.write_load(f, "i32.load16_s", result, ptr, offset),
            Instr::I32Load16U { result, ptr, offset } => self.write_load(f, "i32.load16_u", result, ptr, offset),
            Instr::I64Load8S { result, ptr, offset } => self.write_load(f, "i64.load8_s", result, ptr, offset),
            Instr::I64Load8U { result, ptr, offset } => self.write_load(f, "i64.load8_u", result, ptr, offset),
            Instr::I64Load16S { result, ptr, offset } => self.write_load(f, "i64.load16_s", result, ptr, offset),
            Instr::I64Load16U { result, ptr, offset } => self.write_load(f, "i64.load16_u", result, ptr, offset),
            Instr::I64Load32S { result, ptr, offset } => self.write_load(f, "i64.load32_s", result, ptr, offset),
            Instr::I64Load32U { result, ptr, offset } => self.write_load(f, "i64.load32_u", result, ptr, offset),
            Instr::I32Store { ptr, offset, value } => self.write_store(f, "i32.store", ptr, offset, value),
            Instr::I64Store { ptr, offset, value } => self.write_store(f, "i64.store", ptr, offset, value),
            Instr::F32Store { ptr, offset, value } => self.write_store(f, "f32.store", ptr, offset, value),
            Instr::F64Store { ptr, offset, value } => self.write_store(f, "f64.store", ptr, offset, value),
            Instr::I32Store8 { ptr, offset, value } => self.write_store(f, "i32.store8", ptr, offset, value),
            Instr::I32Store16 { ptr, offset, value } => self.write_store(f, "i32.store16", ptr, offset, value),
            Instr::I64Store8 { ptr, offset, value } => self.write_store(f, "i64.store8", ptr, offset, value),
            Instr::I64Store16 { ptr, offset, value } => self.write_store(f, "i64.store16", ptr, offset, value),
            Instr::I64Store32 { ptr, offset, value } => self.write_store(f, "i64.store32", ptr, offset, value),
            Instr::MemorySize { result } => {
                write!(f, "{} <- memory.size", DisplayExecRegister::from(result))
            }
            Instr::MemoryGrow { result, amount } => {
                write!(
                    f,
                    "{} <- memory.grow {}",
                    DisplayExecRegister::from(result),
                    DisplayExecProvider::new(res, amount)
                )
            }
            Instr::I32Eq { result, lhs, rhs } => self.write_binary(f, "i32.eq", result, lhs, rhs),
            Instr::I32Ne { result, lhs, rhs } => self.write_binary(f, "i32.ne", result, lhs, rhs),
            Instr::I32LtS { result, lhs, rhs } => self.write_binary(f, "i32.lt_s", result, lhs, rhs),
            Instr::I32LtU { result, lhs, rhs } => self.write_binary(f, "i32.lt_u", result, lhs, rhs),
            Instr::I32GtS { result, lhs, rhs } => self.write_binary(f, "i32.gt_s", result, lhs, rhs),
            Instr::I32GtU { result, lhs, rhs } => self.write_binary(f, "i32.gt_u", result, lhs, rhs),
            Instr::I32LeS { result, lhs, rhs } => self.write_binary(f, "i32.le_s", result, lhs, rhs),
            Instr::I32LeU { result, lhs, rhs } => self.write_binary(f, "i32.le_u", result, lhs, rhs),
            Instr::I32GeS { result, lhs, rhs } => self.write_binary(f, "i32.ge_s", result, lhs, rhs),
            Instr::I32GeU { result, lhs, rhs } => self.write_binary(f, "i32.ge_u", result, lhs, rhs),
            Instr::I64Eq { result, lhs, rhs } => self.write_binary(f, "i64.eq", result, lhs, rhs),
            Instr::I64Ne { result, lhs, rhs } => self.write_binary(f, "i64.ne", result, lhs, rhs),
            Instr::I64LtS { result, lhs, rhs } => self.write_binary(f, "i64.lt_s", result, lhs, rhs),
            Instr::I64LtU { result, lhs, rhs } => self.write_binary(f, "i64.lt_u", result, lhs, rhs),
            Instr::I64GtS { result, lhs, rhs } => self.write_binary(f, "i64.gt_s", result, lhs, rhs),
            Instr::I64GtU { result, lhs, rhs } => self.write_binary(f, "i64.gt_u", result, lhs, rhs),
            Instr::I64LeS { result, lhs, rhs } => self.write_binary(f, "i64.le_s", result, lhs, rhs),
            Instr::I64LeU { result, lhs, rhs } => self.write_binary(f, "i64.le_u", result, lhs, rhs),
            Instr::I64GeS { result, lhs, rhs } => self.write_binary(f, "i64.ge_s", result, lhs, rhs),
            Instr::I64GeU { result, lhs, rhs } => self.write_binary(f, "i64.ge_u", result, lhs, rhs),
            Instr::F32Eq { result, lhs, rhs } => self.write_binary(f, "f32.eq", result, lhs, rhs),
            Instr::F32Ne { result, lhs, rhs } => self.write_binary(f, "f32.ne", result, lhs, rhs),
            Instr::F32Lt { result, lhs, rhs } => self.write_binary(f, "f32.lt", result, lhs, rhs),
            Instr::F32Gt { result, lhs, rhs } => self.write_binary(f, "f32.gt", result, lhs, rhs),
            Instr::F32Le { result, lhs, rhs } => self.write_binary(f, "f32.le", result, lhs, rhs),
            Instr::F32Ge { result, lhs, rhs } => self.write_binary(f, "f32.ge", result, lhs, rhs),
            Instr::F64Eq { result, lhs, rhs } => self.write_binary(f, "f64.eq", result, lhs, rhs),
            Instr::F64Ne { result, lhs, rhs } => self.write_binary(f, "f64.ne", result, lhs, rhs),
            Instr::F64Lt { result, lhs, rhs } => self.write_binary(f, "f64.lt", result, lhs, rhs),
            Instr::F64Gt { result, lhs, rhs } => self.write_binary(f, "f64.gt", result, lhs, rhs),
            Instr::F64Le { result, lhs, rhs } => self.write_binary(f, "f64.le", result, lhs, rhs),
            Instr::F64Ge { result, lhs, rhs } => self.write_binary(f, "f64.ge", result, lhs, rhs),
            Instr::I32Clz { result, input } => self.write_unary(f, "i32.clz", result, input),
            Instr::I32Ctz { result, input } => self.write_unary(f, "i32.ctz", result, input),
            Instr::I32Popcnt { result, input } => self.write_unary(f, "i32.popcnt", result, input),
            Instr::I32Add { result, lhs, rhs } => self.write_binary(f, "i32.add", result, lhs, rhs),
            Instr::I32Sub { result, lhs, rhs } => self.write_binary(f, "i32.sub", result, lhs, rhs),
            Instr::I32Mul { result, lhs, rhs } => self.write_binary(f, "i32.mul", result, lhs, rhs),
            Instr::I32DivS { result, lhs, rhs } => self.write_binary(f, "i32.div_s", result, lhs, rhs),
            Instr::I32DivU { result, lhs, rhs } => self.write_binary(f, "i32.div_u", result, lhs, rhs),
            Instr::I32RemS { result, lhs, rhs } => self.write_binary(f, "i32.rem_s", result, lhs, rhs),
            Instr::I32RemU { result, lhs, rhs } => self.write_binary(f, "i32.rem_u", result, lhs, rhs),
            Instr::I32And { result, lhs, rhs } => self.write_binary(f, "i32.and", result, lhs, rhs),
            Instr::I32Or { result, lhs, rhs } => self.write_binary(f, "i32.or", result, lhs, rhs),
            Instr::I32Xor { result, lhs, rhs } => self.write_binary(f, "i32.xor", result, lhs, rhs),
            Instr::I32Shl { result, lhs, rhs } => self.write_binary(f, "i32.shl", result, lhs, rhs),
            Instr::I32ShrS { result, lhs, rhs } => self.write_binary(f, "i32.shr_s", result, lhs, rhs),
            Instr::I32ShrU { result, lhs, rhs } => self.write_binary(f, "i32.shr_u", result, lhs, rhs),
            Instr::I32Rotl { result, lhs, rhs } => self.write_binary(f, "i32.rotl", result, lhs, rhs),
            Instr::I32Rotr { result, lhs, rhs } => self.write_binary(f, "i32.rotr", result, lhs, rhs),
            Instr::I64Clz { result, input } => self.write_unary(f, "i64.clz", result, input),
            Instr::I64Ctz { result, input } => self.write_unary(f, "i64.ctz", result, input),
            Instr::I64Popcnt { result, input } => self.write_unary(f, "i64.popcnt", result, input),
            Instr::I64Add { result, lhs, rhs } => self.write_binary(f, "i64.add", result, lhs, rhs),
            Instr::I64Sub { result, lhs, rhs } => self.write_binary(f, "i64.sub", result, lhs, rhs),
            Instr::I64Mul { result, lhs, rhs } => self.write_binary(f, "i64.mul", result, lhs, rhs),
            Instr::I64DivS { result, lhs, rhs } => self.write_binary(f, "i64.div_s", result, lhs, rhs),
            Instr::I64DivU { result, lhs, rhs } => self.write_binary(f, "i64.div_u", result, lhs, rhs),
            Instr::I64RemS { result, lhs, rhs } => self.write_binary(f, "i64.rem_s", result, lhs, rhs),
            Instr::I64RemU { result, lhs, rhs } => self.write_binary(f, "i64.rem_u", result, lhs, rhs),
            Instr::I64And { result, lhs, rhs } => self.write_binary(f, "i64.and", result, lhs, rhs),
            Instr::I64Or { result, lhs, rhs } => self.write_binary(f, "i64.or", result, lhs, rhs),
            Instr::I64Xor { result, lhs, rhs } => self.write_binary(f, "i64.xor", result, lhs, rhs),
            Instr::I64Shl { result, lhs, rhs } => self.write_binary(f, "i64.shl", result, lhs, rhs),
            Instr::I64ShrS { result, lhs, rhs } => self.write_binary(f, "i64.shr_s", result, lhs, rhs),
            Instr::I64ShrU { result, lhs, rhs } => self.write_binary(f, "i64.shr_u", result, lhs, rhs),
            Instr::I64Rotl { result, lhs, rhs } => self.write_binary(f, "i64.rotl", result, lhs, rhs),
            Instr::I64Rotr { result, lhs, rhs } => self.write_binary(f, "i64.rotr", result, lhs, rhs),
            Instr::F32Abs { result, input } => self.write_unary(f, "f32.abs", result, input),
            Instr::F32Neg { result, input } => self.write_unary(f, "f32.neg", result, input),
            Instr::F32Ceil { result, input } => self.write_unary(f, "f32.ceil", result, input),
            Instr::F32Floor { result, input } => self.write_unary(f, "f32.floor", result, input),
            Instr::F32Trunc { result, input } => self.write_unary(f, "f32.trunc", result, input),
            Instr::F32Nearest { result, input } => self.write_unary(f, "f32.nearest", result, input),
            Instr::F32Sqrt { result, input } => self.write_unary(f, "f32.sqrt", result, input),
            Instr::F32Add { result, lhs, rhs } => self.write_binary(f, "f32.add", result, lhs, rhs),
            Instr::F32Sub { result, lhs, rhs } => self.write_binary(f, "f32.sub", result, lhs, rhs),
            Instr::F32Mul { result, lhs, rhs } => self.write_binary(f, "f32.mul", result, lhs, rhs),
            Instr::F32Div { result, lhs, rhs } => self.write_binary(f, "f32.div", result, lhs, rhs),
            Instr::F32Min { result, lhs, rhs } => self.write_binary(f, "f32.min", result, lhs, rhs),
            Instr::F32Max { result, lhs, rhs } => self.write_binary(f, "f32.max", result, lhs, rhs),
            Instr::F32Copysign { result, lhs, rhs } => self.write_binary(f, "f32.copysign", result, lhs, rhs),
            Instr::F64Abs { result, input } => self.write_unary(f, "f64.abs", result, input),
            Instr::F64Neg { result, input } => self.write_unary(f, "f64.neg", result, input),
            Instr::F64Ceil { result, input } => self.write_unary(f, "f64.ceil", result, input),
            Instr::F64Floor { result, input } => self.write_unary(f, "f64.floor", result, input),
            Instr::F64Trunc { result, input } => self.write_unary(f, "f64.trunc", result, input),
            Instr::F64Nearest { result, input } => self.write_unary(f, "f64.nearest", result, input),
            Instr::F64Sqrt { result, input } => self.write_unary(f, "f64.sqrt", result, input),
            Instr::F64Add { result, lhs, rhs } => self.write_binary(f, "f64.add", result, lhs, rhs),
            Instr::F64Sub { result, lhs, rhs } => self.write_binary(f, "f64.sub", result, lhs, rhs),
            Instr::F64Mul { result, lhs, rhs } => self.write_binary(f, "f64.mul", result, lhs, rhs),
            Instr::F64Div { result, lhs, rhs } => self.write_binary(f, "f64.div", result, lhs, rhs),
            Instr::F64Min { result, lhs, rhs } => self.write_binary(f, "f64.min", result, lhs, rhs),
            Instr::F64Max { result, lhs, rhs } => self.write_binary(f, "f64.max", result, lhs, rhs),
            Instr::F64Copysign { result, lhs, rhs } => self.write_binary(f, "f64.copysign", result, lhs, rhs),
            Instr::I32WrapI64 { result, input } => self.write_unary(f, "i32.wrap_i64", result, input),
            Instr::I32TruncSF32 { result, input } => self.write_unary(f, "i32.trunc_f32_s", result, input),
            Instr::I32TruncUF32 { result, input } => self.write_unary(f, "i32.trunc_f32_u", result, input),
            Instr::I32TruncSF64 { result, input } => self.write_unary(f, "i32.trunc_f64_s", result, input),
            Instr::I32TruncUF64 { result, input } => self.write_unary(f, "i32.trunc_f64_u", result, input),
            Instr::I64ExtendSI32 { result, input } => self.write_unary(f, "i64.extend_i32_s", result, input),
            Instr::I64ExtendUI32 { result, input } => self.write_unary(f, "i64.extend_i32_u", result, input),
            Instr::I64TruncSF32 { result, input } => self.write_unary(f, "i64.trunc_f32_s", result, input),
            Instr::I64TruncUF32 { result, input } => self.write_unary(f, "i64.trunc_f32_u", result, input),
            Instr::I64TruncSF64 { result, input } => self.write_unary(f, "i64.trunc_f64_s", result, input),
            Instr::I64TruncUF64 { result, input } => self.write_unary(f, "i64.trunc_f64_u", result, input),
            Instr::F32ConvertSI32 { result, input } => self.write_unary(f, "f32.convert_i32_s", result, input),
            Instr::F32ConvertUI32 { result, input } => self.write_unary(f, "f32.convert_i32_u", result, input),
            Instr::F32ConvertSI64 { result, input } => self.write_unary(f, "f32.convert_i64_s", result, input),
            Instr::F32ConvertUI64 { result, input } => self.write_unary(f, "f32.convert_i64_u", result, input),
            Instr::F32DemoteF64 { result, input } => self.write_unary(f, "f32.demote_f64", result, input),
            Instr::F64ConvertSI32 { result, input } => self.write_unary(f, "f64.convert_i32_s", result, input),
            Instr::F64ConvertUI32 { result, input } => self.write_unary(f, "f64.convert_i32_u", result, input),
            Instr::F64ConvertSI64 { result, input } => self.write_unary(f, "f64.convert_i64_s", result, input),
            Instr::F64ConvertUI64 { result, input } => self.write_unary(f, "f64.convert_i64_u", result, input),
            Instr::F64PromoteF32 { result, input } => self.write_unary(f, "f64.promote_f32", result, input),
            Instr::I32Extend8S { result, input } => self.write_unary(f, "i32.extend8_s", result, input),
            Instr::I32Extend16S { result, input } => self.write_unary(f, "i32.extend16_s", result, input),
            Instr::I64Extend8S { result, input } => self.write_unary(f, "i64.extend8_s", result, input),
            Instr::I64Extend16S { result, input } => self.write_unary(f, "i64.extend16_s", result, input),
            Instr::I64Extend32S { result, input } => self.write_unary(f, "i64.extend32_s", result, input),
            Instr::I32TruncSatF32S { result, input } => self.write_unary(f, "i32.trunc_sat_f32_s", result, input),
            Instr::I32TruncSatF32U { result, input } => self.write_unary(f, "i32.trunc_sat_f32_u", result, input),
            Instr::I32TruncSatF64S { result, input } => self.write_unary(f, "i32.trunc_sat_f64_s", result, input),
            Instr::I32TruncSatF64U { result, input } => self.write_unary(f, "i32.trunc_sat_f64_u", result, input),
            Instr::I64TruncSatF32S { result, input } => self.write_unary(f, "i64.trunc_sat_f32_s", result, input),
            Instr::I64TruncSatF32U { result, input } => self.write_unary(f, "i64.trunc_sat_f32_u", result, input),
            Instr::I64TruncSatF64S { result, input } => self.write_unary(f, "i64.trunc_sat_f64_s", result, input),
            Instr::I64TruncSatF64U { result, input } => self.write_unary(f, "i64.trunc_sat_f64_u", result, input),
        }
    }
}