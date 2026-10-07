// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;import java.util.*;import com.fasterxml.jackson.databind.ObjectMapper;
public class DotVertexLifecycleProbe extends DotCompetitionProbe {
 static Object state(Object sig,Object chord,Object measure,List<Object> dots)throws Exception{
  List<Object> ds=new ArrayList<>();for(Object d:dots)ds.add(obj("removed",call(d,"isRemoved"),"abnormal",call(d,"isAbnormal"),"measure_member",((Collection<?>)call(measure,"getAugmentationDots")).contains(d)));
  return obj("dots",ds,"chord_dots",call(chord,"getDotsNumber"),"chord_intrinsic_duration",fraction(call(chord,"getDurationSansTuplet")));
 }
 public static void main(String[] args)throws Exception{
  List<Object> cases=new ArrayList<>();for(int i=0;i<4;i++){
   boolean multi=i>=2,warm=i==1,manual=i==3;
   Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[multi?1:0])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig"),chord=null,measure=null;List<Object> dots=new ArrayList<>(),heads=new ArrayList<>();
   if(multi){
    for(Object n:(Collection<?>)call(sig,"vertexSet"))if(n.getClass().getSimpleName().equals("HeadChordInter")&&((Collection<?>)call(n,"getNotes")).size()==2){chord=n;break;}
    measure=call(chord,"getMeasure");heads.addAll((Collection<?>)call(chord,"getNotes"));
    for(int j=0;j<2;j++){java.awt.Rectangle b=(java.awt.Rectangle)call(heads.get(j),"getBounds");Object d=dot(sheet,sig,heads.get(j),b.x+b.width+12,b.y-5);dots.add(d);edge(sig,d,heads.get(j),"AugmentationRelation",.7);call(measure,"addInter",d);}
    if(manual)call(dots.get(1),"setManual",true);
   }else{
    Object dd=null;for(Object e:(Collection<?>)call(sig,"edgeSet"))if(e.getClass().getSimpleName().equals("DoubleDotRelation")){dd=e;break;}
    dots.add(call(sig,"getEdgeTarget",dd));dots.add(call(sig,"getEdgeSource",dd));
    for(Object e:(Collection<?>)call(sig,"edgesOf",dots.get(0)))if(e.getClass().getSimpleName().equals("AugmentationRelation")){Object h=call(sig,"getEdgeTarget",e);heads.add(h);chord=call(h,"getChord");break;}
    measure=call(chord,"getMeasure");if(warm)for(Object d:dots)call(d,"getPart");
   }
   call(chord,"countDots");Object before=state(sig,chord,measure,dots);call(dots.get(0),"remove");Object after=state(sig,chord,measure,dots);Object reducer=make("sig.SigReducer",sys,true),mods=invoke(reducer,"checkAugmentationDots");Object checked=state(sig,chord,measure,dots);boolean physical=true;for(Object d:dots)physical&=((Collection<?>)call(call(sheet,"getGlyphIndex"),"getEntities")).contains(call(d,"getGlyph"));
   cases.add(obj("kind",multi?"declared_multi":"actual_chain","part_getters_warmed",warm,"remaining_dot_manual",manual,"chord_bounds",rect(call(chord,"getBounds")),"head_bounds",heads.stream().map(h->{try{return rect(call(h,"getBounds"));}catch(Exception e){throw new RuntimeException(e);}}).toList(),"dot_bounds",dots.stream().map(d->{try{return rect(call(d,"getBounds"));}catch(Exception e){throw new RuntimeException(e);}}).toList(),"before",before,"after_first_dot_vertex_remove",after,"target_modifications",mods,"after_target_check",checked,"repeat_modifications",invoke(reducer,"checkAugmentationDots"),"physical_glyphs_retained",physical));
  }
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[2]).toFile(),obj("original_source","9e1e55cd2746037d059345881c53e6a6754bffbd","scope","actual successful double-dot vertex loss cold/warm, plus declared dots on actual two-member HeadChord automatic/manual count callbacks; not new automatic image success","cases",cases));System.exit(0);
 }
}
