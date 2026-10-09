// SPDX-License-Identifier: AGPL-3.0-or-later
import java.net.*;
import java.nio.file.*;
import java.io.*;
import java.security.ProtectionDomain;
import java.util.*;
import jdk.internal.org.objectweb.asm.*;
public class SlurLifecycleLoader extends URLClassLoader {
 public static final List<String> transformed = Collections.synchronizedList(new ArrayList<>());
 public SlurLifecycleLoader(URL[] urls){super(urls,ClassLoader.getPlatformClassLoader());}
 protected Class<?> findClass(String name)throws ClassNotFoundException{
  if(name.startsWith("org.audiveris.omr.")) {
   URL resource=findResource(name.replace('.','/')+".class");
   if(resource==null)throw new ClassNotFoundException(name);
   try(InputStream input=resource.openStream()) {byte[] original=input.readAllBytes();byte[] changed=transform(name.replace('.','/'),original);return defineClass(name,changed==null?original:changed,0,changed==null?original.length:changed.length);}
   catch(IOException e){throw new ClassNotFoundException(name,e);}
  }
  return super.findClass(name);
 }
 static byte[] transform(String name,byte[] bytes) {
    String prefix="org/audiveris/omr/";
    if(!Set.of(prefix+"sig/inter/SlurInter",prefix+"sig/inter/AbstractInter",prefix+"sig/relation/SlurHeadRelation",prefix+"sheet/Part",prefix+"sheet/curve/SlursBuilder",prefix+"sheet/curve/ClumpPruner$ClumpLinker").contains(name))return null;
    ClassReader reader=new ClassReader(bytes);ClassWriter writer=new ClassWriter(reader,ClassWriter.COMPUTE_MAXS);
    reader.accept(new ClassVisitor(Opcodes.ASM9,writer) {
     public MethodVisitor visitMethod(int access,String method,String desc,String signature,String[] exceptions) {
      MethodVisitor parent=super.visitMethod(access,method,desc,signature,exceptions);
      boolean selected=(name.endsWith("/SlurInter")&&method.equals("added"))||(name.endsWith("/AbstractInter")&&method.equals("setPart"))||(name.endsWith("/SlurHeadRelation")&&method.equals("added"))||(name.endsWith("/Part")&&method.equals("addSlur"))||(name.endsWith("/SlursBuilder")&&method.equals("dispatchToParts"))||(name.endsWith("/ClumpPruner$ClumpLinker")&&method.equals("doLink"));
      if(!selected)return parent;
      transformed.add(name+"."+method+desc);
      return new MethodVisitor(Opcodes.ASM9,parent) {
       void hook(String point) {super.visitLdcInsn(name.substring(prefix.length())+"."+method+"."+point);super.visitVarInsn(Opcodes.ALOAD,0);if(desc.startsWith("()"))super.visitInsn(Opcodes.ACONST_NULL);else super.visitVarInsn(Opcodes.ALOAD,1);super.visitMethodInsn(Opcodes.INVOKESTATIC,"SlurLifecycleObserver","capture","(Ljava/lang/String;Ljava/lang/Object;Ljava/lang/Object;)V",false);}
       public void visitCode(){super.visitCode();hook("entry");}
       public void visitInsn(int opcode){if(opcode>=Opcodes.IRETURN&&opcode<=Opcodes.RETURN)hook("return");super.visitInsn(opcode);}
      };
     }
    },0);return writer.toByteArray();
 }
 public static void main(String[] args)throws Exception {
  List<URL> urls=new ArrayList<>();try(var files=Files.list(Path.of(args[0]))){for(Path p:files.filter(p->p.toString().endsWith(".jar")).sorted().toList())urls.add(p.toUri().toURL());}urls.add(Path.of(args[1]).toUri().toURL());
  try(var loader=new SlurLifecycleLoader(urls.toArray(URL[]::new))){Thread.currentThread().setContextClassLoader(loader);Class<?> observer=loader.loadClass("SlurLifecycleObserver");observer.getField("transformedMethods").set(null,transformed);observer.getMethod("main",String[].class).invoke(null,(Object)new String[]{args[2],args[3]});}
 }
}
