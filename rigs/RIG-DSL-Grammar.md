# RIG-DSL 完整语法范式

基于项目解析器代码提取的完整语法规范。

## EBNF 语法定义

```ebnf
// 顶层模块定义
RigModule := (RigTemplate | RigClass | RigExternOP)* ;

// 外部操作定义
RigExternOP := "extern" "class" Ident ";" ;

// 类定义
RigClass := "class" Ident RigInstFieldList ;

// 模板定义  
RigTemplate := "template" IdentArray Ident "{" (RigInstField ",")* ";" RigTemplateImpls "}" ;

// 指令字段列表
RigInstFieldList := "{" "}"
                  | "{" RigInstField ("," RigInstField)* (",")? "}" ;

// 指令字段
RigInstField := RigInRegs | RigOutRegs | RigInsts | RigProperties ;

RigInRegs := "in" ":" RigOperandDeclList ;
RigOutRegs := "out" ":" RigOperandDeclList ;
RigInsts := "insts" ":" IdentArray ;
RigProperties := "props" ":" RigPropList ;

// 操作数声明列表
RigOperandDeclList := "{" "}"
                    | "{" RigOperandDecl ("," RigOperandDecl)* (",")? "}" ;

// 操作数声明
RigOperandDecl := Ident ":" RigOperand ;

// 操作数类型
RigOperand := Ident                                    // 模板参数 T
            | "GPR32" | "GPR64" | "GSP32" | "GSP64" | "XSP" | "WSP"
            | "GPR" "(" LitInt "," IdentArray ")"       // 可配置 GPR
            | "PState" | "PC"
            | "FPR32" | "FPR64"
            | "FPR" "(" LitInt "," IdentArray ")"       // 可配置 FPR
            | "Imm32" | "Imm64" | "ImmCalc" | "ImmLogic" | "ImmSMax" | "ImmUMax"
            | "ImmShift" | "ImmLSP64" | "ImmLSP32 | "ImmCCmp" | "ImmMov" | "ImmFMov"
            | "Label" | "Global" | "Symbol" | "SwitchTab" | "Any" ;

// 属性列表
RigPropList := "{" (RigProp)* "}" ;

// 属性定义
RigProp := Ident ":" Type "=" Expr ";"                                    // 简单属性
         | Ident ":" Type "{" (GetProp | SetProp | DefaultProp)+ "}" (";")?  // 复杂属性
         ;

GetProp := "get" ";" | "get" ExprBlock ;
SetProp := "set" ";" | "set" ExprBlock ;
DefaultProp := "default" "=" Expr ";" ;

// 模板实现列表
RigTemplateImpls := "impl" "{" "}"
                  | "impl" "{" RigTemplateImpl ("," RigTemplateImpl)* (",")? "}" ;

// 模板实现项
RigTemplateImpl := IdentArray "=>" RigTemplateImplItem ;

// 模板实现内容
RigTemplateImplItem := Ident RigInstFieldList ;

// 标识符数组
IdentArray := "[" "]"
            | "[" Ident ("," Ident)* (",")? "]" ;

// 基础元素
Ident := [a-zA-Z][a-zA-Z0-9_]* ;
LitInt := [0-9]+ ;
Type := Rust类型表达式 ;
Expr := Rust表达式 ;
ExprBlock := Rust块表达式 ;
```

## 语法元素分类

### 1. 关键字 (Keywords)
- **结构定义**: `class`, `template`, `extern`
- **字段类型**: `in`, `out`, `insts`, `props`
- **模板实现**: `impl`
- **属性操作**: `get`, `set`, `default`

### 2. 操作数类型 (Operand Types)

#### 通用寄存器
- `GPR32`: 32位通用寄存器
- `GPR64`: 64位通用寄存器
- `GSP32`: 32位通用寄存器或栈指针
- `GSP64`: 64位通用寄存器或栈指针
- `XSP`: 64位栈指针
- `WSP`: 32位栈指针
- `GPR(bits, flags)`: 可配置的通用寄存器

#### 浮点寄存器
- `FPR32`: 32位浮点寄存器
- `FPR64`: 64位浮点寄存器
- `FPR(bits, flags)`: 可配置的浮点寄存器

#### 立即数类型
- `Imm32`: 32位立即数
- `Imm64`: 64位立即数
- `ImmCalc`: 计算用立即数
- `ImmLogic`: 逻辑运算立即数
- `ImmSMax`: 最大/最小值立即数
- `ImmUMax`: 最大/最小值立即数
- `ImmShift`: 移位操作立即数
- `ImmLSP32`: 加载指令立即数, 32 位变体
- `ImmLSP64`: 加载指令立即数, 64 位变体
- `ImmCCmp`: 条件比较立即数
- `ImmMov`: 移动指令立即数
- `ImmFMov32`: 浮点移动立即数, 32 位浮点变体
- `ImmFMov64`: 浮点移动立即数, 64 位浮点变体

#### 特殊操作数
- `Label`: 标签引用
- `Global`: 全局符号引用
- `Symbol`: 符号引用 (标签、全局符号或跳转表)
- `SwitchTab`: 跳转表
- `PState`: 处理器状态寄存器
- `PC`: 程序计数器
- `Any`: 任意操作数类型 (通常用于MIR伪指令)

### 3. 指令名称模式

指令名称通常遵循以下模式：
- 以大写字母开头
- 可包含数字和下划线
- 常见后缀：`R` (寄存器操作), `I` (立即数操作)
- 示例：`Add64R`, `Sub32I`, `BCond`, `SMULL`

### 4. 语法结构

#### 类定义示例
```rig
class CondBr {
    in: {
        label: Label,
        csr:   PState,
    },
    insts: [ BCond, BCCond ],
    props: {
        cond: MirCondFlag = MirCondFlag::AL;
    }
}
```

#### 模板定义示例
```rig
template[Lhs, Rhs] CompareInsts {
    in: {
        rn:  Lhs,
        rhs: Rhs,
    },
    out: { csr: PState, };
    impl {
        [GPR64, GPR64] => ICmp64R {
            insts: [ ICmp64R, ICmn64R ],
            props: {
                rm_op: Option<RegOP> = None;
            }
        },
        [GPR32, GPR32] => ICmp32R {
            insts: [ ICmp32R, ICmn32R ],
        },
    }
}
```

#### 外部类声明示例
```rig
extern class MirCall;
extern class MirReturn;
```

## 注释

- 行注释：`// 注释内容`
- 块注释：`/* 注释内容 */`

## 语法特点

1. **类型安全**: 操作数类型系统确保编译时类型检查
2. **模板化**: 支持泛型模板，实现代码复用
3. **属性系统**: 支持自定义属性和访问器
4. **指令映射**: 每个类可映射到多个机器指令
5. **Rust集成**: 属性值和表达式使用 Rust 语法
