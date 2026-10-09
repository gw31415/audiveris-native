// SPDX-License-Identifier: AGPL-3.0-or-later
import java.net.*;
import java.nio.file.*;
import java.io.*;
import java.security.ProtectionDomain;
import java.util.*;
import jdk.internal.org.objectweb.asm.*;
public class RestRemovalRewriter {
 public static final List<String> transformed = Collections.synchronizedList(new ArrayList<>());
 static byte[] transform(String name,byte[] bytes) {
    String prefix="org/audiveris/omr/";
    if(!Set.of(prefix+"sig/inter/AbstractNoteInter",prefix+"sig/inter/AbstractChordInter",prefix+"sig/inter/AbstractInter",prefix+"sheet/Staff",prefix+"sheet/rhythm/Measure",prefix+"sheet/rhythm/Voice",prefix+"sig/relation/Containment").contains(name))return null;
    ClassReader reader=new ClassReader(bytes);ClassWriter writer=new ClassWriter(reader,ClassWriter.COMPUTE_MAXS);
    reader.accept(new ClassVisitor(Opcodes.ASM9,writer) {
     public MethodVisitor visitMethod(int access,String method,String desc,String signature,String[] exceptions) {
      MethodVisitor parent=super.visitMethod(access,method,desc,signature,exceptions);
      boolean selected=(name.endsWith("/AbstractNoteInter")&&method.equals("remove")&&desc.equals("(Z)V"))||(name.endsWith("/AbstractChordInter")&&((method.equals("remove")&&desc.equals("(Z)V"))||method.equals("invalidateCache")))||(name.endsWith("/AbstractInter")&&method.equals("remove")&&desc.equals("(Z)V"))||(name.endsWith("/Staff")&&method.equals("removeNote"))||(name.endsWith("/Measure")&&(method.equals("removeInter")||method.equals("removeVoiceChord")))||(name.endsWith("/Voice")&&method.equals("removeChord"))||(name.endsWith("/Containment")&&method.equals("removed"));
      if(!selected)return parent;
      transformed.add(name+"."+method+desc);
      return new MethodVisitor(Opcodes.ASM9,parent) {
       void hook(String point) {super.visitLdcInsn(name.substring(prefix.length())+"."+method+"."+point);super.visitVarInsn(Opcodes.ALOAD,0);if(desc.startsWith("()")||desc.startsWith("(Z"))super.visitInsn(Opcodes.ACONST_NULL);else super.visitVarInsn(Opcodes.ALOAD,1);super.visitMethodInsn(Opcodes.INVOKESTATIC,"RestRemovalClockProbe","capture","(Ljava/lang/String;Ljava/lang/Object;Ljava/lang/Object;)V",false);}
       public void visitCode(){super.visitCode();hook("entry");}
       public void visitInsn(int opcode){if(opcode>=Opcodes.IRETURN&&opcode<=Opcodes.RETURN)hook("return");super.visitInsn(opcode);}
      };
     }
    },0);return writer.toByteArray();
 }
 public static void main(String[] args)throws Exception {
  List<Object> records=new ArrayList<>();try(var jar=new java.util.zip.ZipFile(args[0])) {
   for(String type:List.of("sig/inter/AbstractNoteInter","sig/inter/AbstractChordInter","sig/inter/AbstractInter","sheet/Staff","sheet/rhythm/Measure","sheet/rhythm/Voice","sig/relation/Containment")) {
    String name="org/audiveris/omr/"+type;byte[] original=jar.getInputStream(jar.getEntry(name+".class")).readAllBytes();int minor=((original[4]&255)<<8)|(original[5]&255),major=((original[6]&255)<<8)|(original[7]&255);byte[] parsing=original.clone();parsing[4]=0;parsing[5]=0;parsing[6]=0;parsing[7]=65;
    byte[] result=transform(name,parsing);result[4]=original[4];result[5]=original[5];result[6]=original[6];result[7]=original[7];Path output=Path.of(args[1]).resolve(name+".class");Files.createDirectories(output.getParent());Files.write(output,result);
    var digest=java.security.MessageDigest.getInstance("SHA-256");records.add(PedalRawStaffBookProbe.map("class",name,"original_minor",minor,"original_major",major,"original_sha256",java.util.HexFormat.of().formatHex(digest.digest(original)),"observed_sha256",java.util.HexFormat.of().formatHex(digest.digest(result))));
   }
  }Files.writeString(Path.of(args[2]),PedalRawStaffBookProbe.json(PedalRawStaffBookProbe.map("method_hooks",transformed,"classes",records))+"\n");Files.write(Path.of(args[3]),transformed);
 }
}
