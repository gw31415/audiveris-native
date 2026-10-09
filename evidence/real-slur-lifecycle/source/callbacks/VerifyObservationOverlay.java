// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.*;
import java.util.*;
import java.io.*;
import java.util.zip.*;
import jdk.internal.org.objectweb.asm.*;
import jdk.internal.org.objectweb.asm.tree.*;
import jdk.internal.org.objectweb.asm.util.*;
public class VerifyObservationOverlay {
 static ClassNode node(byte[] original){byte[] b=original.clone();b[4]=0;b[5]=0;b[6]=0;b[7]=65;ClassNode n=new ClassNode(Opcodes.ASM9);new ClassReader(b).accept(n,0);return n;}
 static String text(ClassNode node){StringWriter out=new StringWriter();node.accept(new TraceClassVisitor(new PrintWriter(out)));return out.toString();}
 public static void main(String[] args)throws Exception{
  List<Object> rows=new ArrayList<>();int total=0;
  try(ZipFile jar=new ZipFile(args[0])){for(Path p:Files.walk(Path.of(args[1])).filter(p->p.toString().endsWith(".class")).sorted().toList()){
   String name=Path.of(args[1]).relativize(p).toString();byte[] before=jar.getInputStream(jar.getEntry(name)).readAllBytes(),after=Files.readAllBytes(p);if(!Arrays.equals(Arrays.copyOfRange(before,0,8),Arrays.copyOfRange(after,0,8)))throw new AssertionError("header changed "+name);
   ClassNode a=node(before),b=node(after);int removed=0;
   for(int i=0;i<a.methods.size();i++){MethodNode orig=a.methods.get(i),instrumented=b.methods.get(i);if(!orig.name.equals(instrumented.name)||!orig.desc.equals(instrumented.desc))throw new AssertionError("method order");
    for(AbstractInsnNode insn=instrumented.instructions.getFirst();insn!=null;){AbstractInsnNode next=insn.getNext();if(insn instanceof MethodInsnNode call&&call.owner.equals("SlurLifecycleObserver")&&call.name.equals("capture")){
     AbstractInsnNode arg=insn.getPrevious(),receiver=arg.getPrevious(),tag=receiver.getPrevious();if(!(tag instanceof LdcInsnNode ldc&&ldc.cst instanceof String)||!(receiver instanceof VarInsnNode v&&v.getOpcode()==Opcodes.ALOAD&&v.var==0)||!(arg.getOpcode()==Opcodes.ACONST_NULL||(arg instanceof VarInsnNode a1&&a1.getOpcode()==Opcodes.ALOAD&&a1.var==1))||!call.desc.equals("(Ljava/lang/String;Ljava/lang/Object;Ljava/lang/Object;)V")||call.getOpcode()!=Opcodes.INVOKESTATIC)throw new AssertionError("unexpected hook");
     for(AbstractInsnNode x:List.of(tag,receiver,arg,insn))instrumented.instructions.remove(x);removed++;
    }insn=next;}instrumented.maxStack=orig.maxStack;
   }
   boolean same=text(a).equals(text(b));rows.add(NativeSlurCandidateMathProbe.map("class",name,"removed_observation_calls",removed,"all_original_instructions_frames_fields_methods_equal",same,"classfile_header_preserved",true));if(!same)throw new AssertionError("body changed "+name);total+=removed;
  }}
  Files.writeString(Path.of(args[2]),NativeSlurCandidateMathProbe.json(NativeSlurCandidateMathProbe.map("scope","JDK ASM tree/text comparison of original pinned jar versus executed six-class overlay after removing only exact four-instruction observer calls and normalizing maxStack. Original instruction order, branches, exception/frame tables, annotations, fields and method declarations are equal. Classfile header bytes preserved. Major 69 temporarily set to 65 only during offline ASM reading; actual runtime received original major 69.","classes",rows,"total_inserted_observation_calls",total,"differences",0))+"\n");
 }
}
