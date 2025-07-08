/// 检查该 32 位立即数是否满足逻辑 and 指令的立即数条件;
/// 具体来说, 立即数应该是一段重复循环的位模式.
pub const fn is_logical_imm32(imm32: u32) -> bool {
    use loop_pattern::*;
    // 先检查一些简单的重复模式
    match imm32 {
        // and 指令不接受循环节为 1 位的立即数.
        // 这个可能是因为这玩意可以直接算出来.
        0x0000_0000 | 0xFFFF_FFFF => return false,
        // 2 位循环节的模式
        0x5555_5555 | 0xAAAA_AAAA => return true,
        // 4 位循环节的模式
        0x1111_1111 | 0x2222_2222 | 0x4444_4444 | 0x8888_8888 | 0x3333_3333 | 0x6666_6666
        | 0xCCCC_CCCC | 0x9999_9999 | 0x7777_7777 | 0xEEEE_EEEE | 0xDDDD_DDDD | 0xBBBB_BBBB => {
            return true;
        }
        // 8 位及以上循环节模式情况比较多了, 不能再做这种匹配了.
        _ => {}
    };

    // 检查是否满足循环节为 8 位的模式
    let imm32 = imm32.rotate_right(imm32.trailing_ones());
    let imm32 = imm32.rotate_right(imm32.trailing_zeros());
    is_loop8_pattern32(imm32) || is_loop16_pattern32(imm32) || is_loop32_pattern32(imm32)
}

/// 检查该 64 位立即数是否满足逻辑 and 指令的立即数条件;
/// 具体来说, 立即数应该是一段重复循环的位模式.
pub const fn is_logical_imm64(imm64: u64) -> bool {
    use loop_pattern::*;
    // 先检查一些简单的重复模式
    match imm64 {
        // and 指令不接受循环节为 1 位的立即数.
        // 这个可能是因为这玩意可以直接算出来.
        0x0000_0000_0000_0000 | 0xFFFF_FFFF_FFFF_FFFF => return false,
        // 2 位循环节的模式
        0x5555_5555_5555_5555 | 0xAAAA_AAAA_AAAA_AAAA => return true,
        // 4 位循环节的模式
        0x1111_1111_1111_1111
        | 0x2222_2222_2222_2222
        | 0x4444_4444_4444_4444
        | 0x8888_8888_8888_8888
        | 0x3333_3333_3333_3333
        | 0x6666_6666_6666_6666
        | 0xCCCC_CCCC_CCCC_CCCC
        | 0x9999_9999_9999_9999
        | 0x7777_7777_7777_7777
        | 0xEEEE_EEEE_EEEE_EEEE
        | 0xDDDD_DDDD_DDDD_DDDD
        | 0xBBBB_BBBB_BBBB_BBBB => {
            return true;
        }
        // 8 位及以上循环节模式情况比较多了, 不能再做这种匹配了.
        _ => {}
    };

    // 检查是否满足循环节为 8 位的模式
    let imm64 = imm64.rotate_right(imm64.trailing_ones());
    let imm64 = imm64.rotate_right(imm64.trailing_zeros());
    is_loop8_pattern64(imm64)
        || is_loop16_pattern64(imm64)
        || is_loop32_pattern64(imm64)
        || is_loop64_pattern64(imm64)
}

pub const fn is_calc_imm(imm: u64) -> bool {
    if imm < 4096 {
        // 12 位以内的立即数可以直接计算
        return true;
    }
    if imm & 0xFFF != 0 {
        // 12 位以上的立即数, 需要满足 imm & 0xFFF == 0
        return false;
    }
    let imm = imm >> 12;
    imm < 4096
}

mod loop_pattern {
    pub(super) const fn is_loop8_pattern32(imm: u32) -> bool {
        let imm8 = (imm & 0xFF) as u8;
        if !(imm8 + 1).is_power_of_two() {
            return false;
        }
        let should_be = imm8 as u32;
        let should_be = should_be | (should_be << 8);
        let should_be = should_be | (should_be << 16);
        imm == should_be
    }

    pub(super) const fn is_loop16_pattern32(imm: u32) -> bool {
        let imm16 = (imm & 0xFFFF) as u16;
        if !(imm16 + 1).is_power_of_two() {
            return false;
        }
        let should_be = imm16 as u32;
        let should_be = should_be | (should_be << 16);
        imm == should_be
    }

    pub(super) const fn is_loop32_pattern32(imm: u32) -> bool {
        (imm + 1).is_power_of_two()
    }

    pub(super) const fn is_loop8_pattern64(imm: u64) -> bool {
        let imm8 = (imm & 0xFF) as u8;
        if !(imm8 + 1).is_power_of_two() {
            return false;
        }
        let should_be = imm8 as u64;
        let should_be = should_be | (should_be << 8);
        let should_be = should_be | (should_be << 16);
        let should_be = should_be | (should_be << 32);
        imm == should_be
    }

    pub(super) const fn is_loop16_pattern64(imm: u64) -> bool {
        let imm16 = (imm & 0xFFFF) as u16;
        if !(imm16 + 1).is_power_of_two() {
            return false;
        }
        let should_be = imm16 as u64;
        let should_be = should_be | (should_be << 16);
        let should_be = should_be | (should_be << 32);
        imm == should_be
    }

    pub(super) const fn is_loop32_pattern64(imm: u64) -> bool {
        let imm32 = (imm & 0xFFFFFFFF) as u32;
        if !(imm32 + 1).is_power_of_two() {
            return false;
        }
        let should_be = imm32 as u64;
        let should_be = should_be | (should_be << 32);
        imm == should_be
    }

    pub(super) const fn is_loop64_pattern64(imm: u64) -> bool {
        (imm + 1).is_power_of_two()
    }
}
