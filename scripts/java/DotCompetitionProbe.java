// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;import java.util.*;import com.fasterxml.jackson.databind.ObjectMapper;
public class DotCompetitionProbe extends RookieMappingProbe {
 static Object dot(Object sheet,Object sig,Object rest,int x,int y)throws Exception {
  Object g=physical(x,y,5,4,"VERTICAL",null);g=call(call(sheet,"getGlyphIndex"),"registerOriginal",g);
  Object d=make("sig.inter.AugmentationDotInter",g,.8);call(d,"setStaff",call(rest,"getStaff"));call(sig,"addVertex",d);return d;
 }
 static Object edge(Object sig,Object a,Object b,String kind,double grade)throws Exception {
  Object e=make("sig.relation."+kind);call(e,"setGrade",grade);call(sig,"addEdge",a,b,e);return e;
 }
 static Object state(Object sig,Object chord,Object measure,List<Object> dots,List<Object> edges)throws Exception {
  List<Object> ds=new ArrayList<>(),es=new ArrayList<>();for(Object d:dots)ds.add(obj("removed",call(d,"isRemoved"),"abnormal",call(d,"isAbnormal"),"measure_member",((Collection<?>)call(measure,"getAugmentationDots")).contains(d)));
  Collection<?> live=(Collection<?>)call(sig,"edgeSet");for(Object e:edges)es.add(live.contains(e));
  return obj("dots",ds,"edges_live",es,"chord_dots",call(chord,"getDotsNumber"),"chord_intrinsic_duration",fraction(call(chord,"getDurationSansTuplet")));
 }
 public static void main(String[] args)throws Exception {
  List<Object> cases=new ArrayList<>();String[] kinds={"rest","rest","rest","rest","double"};double[][] grades={{.3,.8},{.8,.8},{0.,0.},{.8,.3},{.3,.8}};
  for(int i=0;i<kinds.length;i++){
   Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig"),rest=null;
   for(Object n:(Collection<?>)call(sig,"vertexSet"))if(n.getClass().getSimpleName().equals("RestInter"))rest=n;
   Object chord=call(rest,"getChord"),measure=call(chord,"getMeasure");
   for(Object n:new ArrayList<>((Collection<?>)call(sig,"vertexSet")))if(n.getClass().getSimpleName().equals("AugmentationDotInter"))call(n,"remove");
   List<Object> dots=new ArrayList<>(),edges=new ArrayList<>();boolean doubled=kinds[i].equals("double");
   if(doubled){Object first=dot(sheet,sig,rest,528,245);dots.add(first);edges.add(edge(sig,first,rest,"AugmentationRelation",.7));}
   for(int j=0;j<2;j++){Object d=dot(sheet,sig,rest,doubled?538+j*3:528+j*3,245+j);dots.add(d);edges.add(edge(sig,d,doubled?dots.get(0):rest,doubled?"DoubleDotRelation":"AugmentationRelation",grades[i][j]));}
   call(chord,"countDots");Object before=state(sig,chord,measure,dots,edges),reducer=make("sig.SigReducer",sys,true);
   int firstTargets=(Integer)invoke(reducer,"checkAugmentationDots");int mods=(Integer)invoke(reducer,doubled?"checkAugmentedDots":"checkAugmentedRests");Object after=state(sig,chord,measure,dots,edges);
   int laterTargets=(Integer)invoke(reducer,"checkAugmentationDots");call(chord,"countDots");Object finalState=state(sig,chord,measure,dots,edges);
   boolean glyphs=true;for(Object d:dots)glyphs&=((Collection<?>)call(call(sheet,"getGlyphIndex"),"getEntities")).contains(call(d,"getGlyph"));
   cases.add(obj("kind",kinds[i],"grades",grades[i],"rest",obj("bounds",rect(call(rest,"getBounds")),"grade",call(rest,"getGrade")),"chord",obj("bounds",rect(call(chord,"getBounds")),"part_id",call(call(chord,"getPart"),"getId"),"staff_id",call(call(rest,"getStaff"),"getId")),"dot_bounds",dots.stream().map(d->{try{return rect(call(d,"getBounds"));}catch(Exception e){throw new RuntimeException(e);}}).toList(),"before",before,"first_target_check_modifications",firstTargets,"competition_modifications",mods,"after_competition_callback",after,"later_target_check_modifications",laterTargets,"after_next_target_check_and_count",finalState,"physical_glyphs_retained",glyphs));
  }
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("original_source","9e1e55cd2746037d059345881c53e6a6754bffbd","scope","declared temporary dots/grades on saved actual original RestChord; direct original rest/double competition, edge callbacks and later target check; not automatic image success","cases",cases));System.exit(0);
 }
}
