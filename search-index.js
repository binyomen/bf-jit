var searchIndex = JSON.parse('{\
"bench":{"doc":"","t":[17,17,17,17,17,3,17,5,11,11,5,11,5,5,11,12,12,11,5,11,11,11],"n":["BAR1_COLOR","BAR2_COLOR","BAR3_COLOR","BAR4_COLOR","BAR5_COLOR","ImplInfo","RESOLUTION","benchmark","borrow","borrow_mut","create_graph","from","graph_results","graph_results_for_file","into","millis","name","new","segmented_value_to_inner","try_from","try_into","type_id"],"q":["bench","","","","","","","","","","","","","","","","","","","","",""],"d":["","","","","","","","","","","","Returns the argument unchanged.","","","Calls <code>U::from(self)</code>.","","","","","","",""],"i":[0,0,0,0,0,0,0,0,6,6,0,6,0,0,6,6,6,6,0,6,6,6],"f":[0,0,0,0,0,0,0,[[1,2,1,1],[[5,[3,4]]]],[[]],[[]],[[1,1],[[5,[4]]]],[[]],[[],[[5,[4]]]],[[1,1,1,1],[[5,[4]]]],[[]],0,0,[[1,2,1,1],[[5,[6,4]]]],[7],[[],5],[[],5],[[],8]],"p":[[15,"str"],[8,"RunFunction"],[15,"u128"],[4,"BfError"],[4,"Result"],[3,"ImplInfo"],[4,"SegmentValue"],[3,"TypeId"]]},\
"opinterp":{"doc":"","t":[0,5,0,13,13,13,13,4,13,13,3,13,13,11,11,11,11,5,11,11,11,11,11,11,12,11,11,12,5,11,11,11,11,11,11,17,5,5,5,5],"n":["parser","run","vm","DecData","DecPtr","IncData","IncPtr","Instruction","JumpIfNotZero","JumpIfZero","Program","Read","Write","borrow","borrow","borrow_mut","borrow_mut","create_jump_table","eq","eq","fmt","fmt","from","from","instructions","into","into","jump_table","parse","try_from","try_from","try_into","try_into","type_id","type_id","MEMORY_SIZE","jump","read","run","write"],"q":["opinterp","","","opinterp::parser","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","opinterp::vm","","","",""],"d":["","","","","","","","","","","","","","","","","","","","","","","Returns the argument unchanged.","Returns the argument unchanged.","","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","","","","","","","","","","","","",""],"i":[0,0,0,7,7,7,7,0,7,7,0,7,7,7,9,7,9,0,7,9,7,9,7,9,9,7,9,9,0,7,9,7,9,7,9,0,0,0,0,0],"f":[0,[[1,2,3],4],0,0,0,0,0,0,0,0,0,0,0,[[]],[[]],[[]],[[]],[5,[[4,[[5,[6]]]]]],[[7,7],8],[[9,9],8],[[7,10],11],[[9,10],11],[[]],[[]],0,[[]],[[]],0,[1,[[4,[9]]]],[[],12],[[],12],[[],12],[[],12],[[],13],[[],13],0,[[8,9,6,6],6],[2,[[4,[14]]]],[[9,2,3],4],[[3,14],4]],"p":[[15,"str"],[8,"Read"],[8,"Write"],[6,"BfResult"],[3,"Vec"],[15,"usize"],[4,"Instruction"],[15,"bool"],[3,"Program"],[3,"Formatter"],[6,"Result"],[4,"Result"],[3,"TypeId"],[15,"u8"]]},\
"opinterp2":{"doc":"","t":[0,5,0,13,13,13,13,13,13,13,13,4,13,13,13,13,4,3,13,13,13,13,11,11,11,11,11,11,11,11,11,11,11,11,11,11,12,11,11,11,5,11,5,11,11,11,11,11,11,11,11,11,12,12,12,12,12,12,12,12,17,5,5,5,5],"n":["parser","run","vm","DecData","DecData","DecPtr","DecPtr","IncData","IncData","IncPtr","IncPtr","Instruction","JumpIfNotZero","JumpIfNotZero","JumpIfZero","JumpIfZero","OpCode","Program","Read","Read","Write","Write","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","eq","eq","eq","fmt","fmt","from","from","from","instructions","into","into","into","parse","set_destination","translate_to_opcodes","try_from","try_from","try_from","try_into","try_into","try_into","type_id","type_id","type_id","count","count","count","count","count","count","destination","destination","MEMORY_SIZE","jump","read","run","write"],"q":["opinterp2","","","opinterp2::parser","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","opinterp2::parser::Instruction","","","","","","","","opinterp2::vm","","","",""],"d":["","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","","","","","","","","","","","","","","","","","","","","","","","","",""],"i":[0,0,0,5,7,5,7,5,7,5,7,0,5,7,5,7,0,0,5,7,5,7,5,7,8,5,7,8,5,7,8,7,8,5,7,8,8,5,7,8,0,7,0,5,7,8,5,7,8,5,7,8,16,17,18,19,20,21,22,23,0,0,0,0,0],"f":[0,[[1,2,3],4],0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,[[]],[[]],[[]],[[]],[[]],[[]],[[5,5],6],[[7,7],6],[[8,8],6],[[7,9],10],[[8,9],10],[[]],[[]],[[]],0,[[]],[[]],[[]],[1,[[4,[8]]]],[[7,11]],[1,[[12,[5]]]],[[],13],[[],13],[[],13],[[],13],[[],13],[[],13],[[],14],[[],14],[[],14],0,0,0,0,0,0,0,0,0,[[6,11,11,11],11],[2,[[4,[15]]]],[[8,2,3],4],[[3,15],4]],"p":[[15,"str"],[8,"Read"],[8,"Write"],[6,"BfResult"],[4,"OpCode"],[15,"bool"],[4,"Instruction"],[3,"Program"],[3,"Formatter"],[6,"Result"],[15,"usize"],[3,"Vec"],[4,"Result"],[3,"TypeId"],[15,"u8"],[13,"IncPtr"],[13,"DecPtr"],[13,"IncData"],[13,"DecData"],[13,"Read"],[13,"Write"],[13,"JumpIfZero"],[13,"JumpIfNotZero"]]},\
"opinterp3":{"doc":"","t":[0,5,0,4,3,13,13,13,13,13,13,13,13,4,13,13,13,13,13,13,13,3,13,13,13,13,13,13,11,11,11,11,11,11,11,11,12,5,5,11,11,11,11,11,11,11,11,11,11,11,11,12,11,11,11,11,5,5,11,11,11,11,11,11,11,11,11,11,11,11,12,12,12,12,12,12,12,12,12,12,12,12,12,12,12,12,12,12,12,17,5,5,5,5],"n":["parser","run","vm","AstNode","AstSeq","DecData","DecData","DecPtr","DecPtr","IncData","IncData","IncPtr","IncPtr","Instruction","JumpBegin","JumpEnd","Loop","MoveData","MoveData","MovePtrUntilZero","MovePtrUntilZero","Program","Read","Read","SetDataToZero","SetDataToZero","Write","Write","borrow","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","borrow_mut","children","compile","create_ast","eq","eq","eq","eq","fmt","fmt","fmt","fmt","from","from","from","from","instructions","into","into","into","into","optimize_loops","parse","try_from","try_from","try_from","try_from","try_into","try_into","try_into","try_into","type_id","type_id","type_id","type_id","amount","amount","forward","forward","seq","amount","amount","count","count","count","count","count","count","count","count","destination","destination","forward","forward","MEMORY_SIZE","jump","read","run","write"],"q":["opinterp3","","","opinterp3::parser","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","opinterp3::parser::AstNode","","","","","opinterp3::parser::Instruction","","","","","","","","","","","","","","opinterp3::vm","","","",""],"d":["","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","",""],"i":[0,0,0,0,0,8,6,8,6,8,6,8,6,0,6,6,8,8,6,8,6,0,8,6,8,6,8,6,8,5,6,10,8,5,6,10,5,0,0,8,5,6,10,8,5,6,10,8,5,6,10,10,8,5,6,10,0,0,8,5,6,10,8,5,6,10,8,5,6,10,17,18,17,18,19,20,21,22,23,24,25,26,27,20,21,28,29,20,21,0,0,0,0,0],"f":[0,[[1,2,3],4],0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],0,[5,[[7,[6]]]],[1,[[4,[5]]]],[[8,8],9],[[5,5],9],[[6,6],9],[[10,10],9],[[8,11],12],[[5,11],12],[[6,11],12],[[10,11],12],[[]],[[]],[[]],[[]],0,[[]],[[]],[[]],[[]],[5,5],[1,[[4,[10]]]],[[],13],[[],13],[[],13],[[],13],[[],13],[[],13],[[],13],[[],13],[[],14],[[],14],[[],14],[[],14],0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,[[9,15,15,15],15],[2,[[4,[16]]]],[[10,2,3],4],[[3,16],4]],"p":[[15,"str"],[8,"Read"],[8,"Write"],[6,"BfResult"],[3,"AstSeq"],[4,"Instruction"],[3,"Vec"],[4,"AstNode"],[15,"bool"],[3,"Program"],[3,"Formatter"],[6,"Result"],[4,"Result"],[3,"TypeId"],[15,"usize"],[15,"u8"],[13,"MovePtrUntilZero"],[13,"MoveData"],[13,"Loop"],[13,"MovePtrUntilZero"],[13,"MoveData"],[13,"IncPtr"],[13,"DecPtr"],[13,"IncData"],[13,"DecData"],[13,"Read"],[13,"Write"],[13,"JumpBegin"],[13,"JumpEnd"]]},\
"simpleinterp":{"doc":"","t":[0,5,0,13,13,13,13,4,13,13,3,13,13,11,11,11,11,11,11,11,11,11,11,12,11,11,5,11,11,11,11,11,11,17,5,5,5,5],"n":["parser","run","vm","DecData","DecPtr","IncData","IncPtr","Instruction","JumpIfNotZero","JumpIfZero","Program","Read","Write","borrow","borrow","borrow_mut","borrow_mut","eq","eq","fmt","fmt","from","from","instructions","into","into","parse","try_from","try_from","try_into","try_into","type_id","type_id","MEMORY_SIZE","jump","read","run","write"],"q":["simpleinterp","","","simpleinterp::parser","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","simpleinterp::vm","","","",""],"d":["","","","","","","","","","","","","","","","","","","","","","Returns the argument unchanged.","Returns the argument unchanged.","","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","","","","","","","","","","","",""],"i":[0,0,0,5,5,5,5,0,5,5,0,5,5,5,7,5,7,5,7,5,7,5,7,7,5,7,0,5,7,5,7,5,7,0,0,0,0,0],"f":[0,[[1,2,3],4],0,0,0,0,0,0,0,0,0,0,0,[[]],[[]],[[]],[[]],[[5,5],6],[[7,7],6],[[5,8],9],[[7,8],9],[[]],[[]],0,[[]],[[]],[1,7],[[],10],[[],10],[[],10],[[],10],[[],11],[[],11],0,[[6,7,12,12],4],[2,[[4,[13]]]],[[7,2,3],4],[[3,13],4]],"p":[[15,"str"],[8,"Read"],[8,"Write"],[6,"BfResult"],[4,"Instruction"],[15,"bool"],[3,"Program"],[3,"Formatter"],[6,"Result"],[4,"Result"],[3,"TypeId"],[15,"usize"],[15,"u8"]]},\
"simplejit":{"doc":"","t":[0,0,5,0,3,3,12,11,11,11,11,12,5,14,12,11,11,11,11,11,12,11,11,11,11,11,11,13,13,13,13,4,13,13,3,13,13,11,11,11,11,11,11,11,11,11,11,12,11,11,5,11,11,11,11,11,11,6,17,3,11,11,11,11,12,11,11,11,11,12,12,11,11,11,11],"n":["compiler","parser","run","runtime","CompiledProgram","LabelPair","begin_label","borrow","borrow","borrow_mut","borrow_mut","buffer","compile","dasm","end_label","from","from","function_ptr","into","into","start","try_from","try_from","try_into","try_into","type_id","type_id","DecData","DecPtr","IncData","IncPtr","Instruction","JumpIfNotZero","JumpIfZero","Program","Read","Write","borrow","borrow","borrow_mut","borrow_mut","eq","eq","fmt","fmt","from","from","instructions","into","into","parse","try_from","try_from","try_into","try_into","type_id","type_id","AsmEntryPoint","MEMORY_SIZE","Runtime","borrow","borrow_mut","from","into","memory","memory_ptr","new","read","run","stdin","stdout","try_from","try_into","type_id","write"],"q":["simplejit","","","","simplejit::compiler","","","","","","","","","","","","","","","","","","","","","","","simplejit::parser","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","simplejit::runtime","","","","","","","","","","","","","","","","",""],"d":["","","","","","","","","","","","","","","","Returns the argument unchanged.","Returns the argument unchanged.","","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","","","","","","","","","","","","","","","","","","","","","","","","","","Returns the argument unchanged.","Returns the argument unchanged.","","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","","","","","","","","","","","","","Returns the argument unchanged.","Calls <code>U::from(self)</code>.","","","","","","","","","","",""],"i":[0,0,0,0,0,0,15,7,15,7,15,7,0,0,15,7,15,7,7,15,7,7,15,7,15,7,15,10,10,10,10,0,10,10,0,10,10,10,5,10,5,10,5,10,5,10,5,5,10,5,0,10,5,10,5,10,5,0,0,0,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6],"f":[0,0,[[1,2,3],4],0,0,0,0,[[]],[[]],[[]],[[]],0,[[5,6],[[4,[7]]]],0,0,[[]],[[]],[7],[[]],[[]],0,[[],8],[[],8],[[],8],[[],8],[[],9],[[],9],0,0,0,0,0,0,0,0,0,0,[[]],[[]],[[]],[[]],[[10,10],11],[[5,5],11],[[10,12],13],[[5,12],13],[[]],[[]],0,[[]],[[]],[1,[[4,[5]]]],[[],8],[[],8],[[],8],[[],8],[[],9],[[],9],0,0,0,[[]],[[]],[[]],[[]],0,[6,14],[[2,3],6],[6,14],[[6,7],4],0,0,[[],8],[[],8],[[],9],[[6,14]]],"p":[[15,"str"],[8,"Read"],[8,"Write"],[6,"BfResult"],[3,"Program"],[3,"Runtime"],[3,"CompiledProgram"],[4,"Result"],[3,"TypeId"],[4,"Instruction"],[15,"bool"],[3,"Formatter"],[6,"Result"],[15,"u8"],[3,"LabelPair"]]},\
"util":{"doc":"","t":[13,13,4,6,13,13,8,13,0,0,5,12,12,12,12,12,13,13,4,6,13,13,13,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,12,12,12,12,12,8,5],"n":["Assembler","Bf","BfError","BfResult","DrawingArea","Io","RunFunction","TryFromInt","error","run","run_main","0","0","0","0","0","Assembler","Bf","BfError","BfResult","DrawingArea","Io","TryFromInt","borrow","borrow_mut","eq","fmt","fmt","from","from","from","from","from","into","provide","to_string","try_from","try_into","type_id","0","0","0","0","0","RunFunction","run_main"],"q":["util","","","","","","","","","","","util::BfError","","","","","util::error","","","","","","","","","","","","","","","","","","","","","","","util::error::BfError","","","","","util::run",""],"d":["","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","Returns the argument unchanged.","Calls <code>U::from(self)</code>.","","","","","","","","","","","",""],"i":[3,3,0,0,3,3,0,3,0,0,0,15,16,17,18,19,3,3,0,0,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,15,16,17,18,19,0,0],"f":[0,0,0,0,0,0,0,0,0,0,[1,2],0,0,0,0,0,0,0,0,0,0,0,0,[[]],[[]],[[3,3],4],[[3,5],6],[[3,5],6],[7,3],[8,3],[9,3],[10,3],[[]],[[]],[11],[[],12],[[],13],[[],13],[[],14],0,0,0,0,0,0,[1,2]],"p":[[8,"RunFunction"],[6,"BfResult"],[4,"BfError"],[15,"bool"],[3,"Formatter"],[6,"Result"],[3,"TryFromIntError"],[3,"Error"],[4,"DrawingAreaErrorKind"],[3,"Assembler"],[3,"Demand"],[3,"String"],[4,"Result"],[3,"TypeId"],[13,"Bf"],[13,"DrawingArea"],[13,"TryFromInt"],[13,"Io"],[13,"Assembler"]]}\
}');
if (typeof window !== 'undefined' && window.initSearch) {window.initSearch(searchIndex)};
if (typeof exports !== 'undefined') {exports.searchIndex = searchIndex};
