// SPDX-License-Identifier: AGPL-3.0-or-later
// Original-owned SIG observations. Controls are separate from full-image export.
import java.nio.file.*;import java.util.*;import com.fasterxml.jackson.databind.ObjectMapper;
public class BeamGroupEpilogProbe extends BeamOriginDependencyProbe {
 static Object system(String path)throws Exception {Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(path));Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");return ((List<?>)call(sheet,"getSystems")).get(0);}
 static Object process(Object sys)throws Exception {Object r=c("sheet.stem.StemsRetriever").getConstructor(c("sheet.SystemInfo")).newInstance(sys);call(r,"process");return r;}
 static Object member(Object n)throws Exception{return obj("kind",n.getClass().getSimpleName(),"bounds",rect(call(n,"getBounds")),"glyph_id",call(call(n,"getGlyph"),"getId"));}
 static Object groups(Object sig)throws Exception {List<Object> out=new ArrayList<>();for(Object n:(Collection<?>)call(sig,"vertexSet"))if(n.getClass().getSimpleName().equals("BeamGroupInter")){List<Object> members=new ArrayList<>();for(Object b:(List<?>)call(n,"getMembers"))members.add(member(b));out.add(obj("bounds",rect(call(n,"getBounds")),"members",members));}return out;}
 public static void main(String[]args)throws Exception {
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
  Object sys=system(args[0]),sig=call(sys,"getSig"),r=process(sys);Object linked=groups(sig);call(r,"finalizeBeams");Object reducer=c("sig.SigReducer").getConstructor(c("sheet.SystemInfo"),boolean.class).newInstance(sys,true);call(reducer,"reduceFoundations");call(c("sig.BeamHeadCleaner").getConstructor(c("sheet.SystemInfo")).newInstance(sys),"process");Object reduced=groups(sig);
  List<Object> cleanup=new ArrayList<>();
  for(String mode:new String[]{"partial","last-member","extensive"}){
   Object s=system(args[0]),g=call(s,"getSig");process(s);Object group=null;for(Object n:(Collection<?>)call(g,"vertexSet"))if(n.getClass().getSimpleName().equals("BeamGroupInter")){group=n;break;}
   List<?>members=new ArrayList<>((List<?>)call(group,"getMembers"));Object before=groups(g); // prime the actual bounds cache
   if(mode.equals("extensive"))call(group,"remove");else if(mode.equals("partial"))call(members.get(0),"remove");else for(Object n:members)call(n,"remove");
   List<Object>removed=new ArrayList<>();for(Object n:members)removed.add(call(n,"isRemoved"));
   cleanup.add(obj("mode",mode,"before",before,"after",groups(g),"group_removed",call(group,"isRemoved"),"members_removed",removed));
  }
  List<Object>grade=new ArrayList<>();
  for(double intrinsic:new double[]{.35-1e-9,.35}){
   Object s=system(args[0]),g=call(s,"getSig"),retriever=process(s),beam=null;int width=-1;
   for(Object n:(Collection<?>)call(g,"vertexSet"))if(n.getClass().getSimpleName().equals("BeamInter")){java.awt.Rectangle b=(java.awt.Rectangle)call(n,"getBounds");if(b.width>width){beam=n;width=b.width;}}
   call(beam,"setGrade",intrinsic);double contextual=intrinsic<.35?.99:.1;call(beam,"setContextualGrade",contextual);call(retriever,"finalizeBeams");int supports=0;for(Object e:(Collection<?>)call(g,"edgeSet"))if(e.getClass().getSimpleName().equals("BeamHeadRelation")&&call(g,"getEdgeSource",e)==beam)supports++;
   grade.add(obj("grade",intrinsic,"declared_contextual_grade",contextual,"is_good",call(beam,"isGood"),"side_supports",supports));
  }
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("native_outputs_used",false,"scope","actual original HEADS -> STEMS/finalize/reduce/BeamHeadCleaner group state; three declared ownership controls; two intrinsic-grade boundaries, independent of full-image export","after_link_stems",linked,"after_reduction_cleaner",reduced,"cleanup_controls",cleanup,"grade_controls",grade));System.exit(0);
 }
}
