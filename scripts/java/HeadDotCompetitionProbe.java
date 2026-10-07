// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;import java.util.*;import com.fasterxml.jackson.databind.ObjectMapper;
public class HeadDotCompetitionProbe extends DotCompetitionProbe {
 static Object state(Object sig,Object chord,List<Object> dots,List<Object> edges)throws Exception{
  List<Object> ds=new ArrayList<>(),liveEdges=new ArrayList<>();for(Object d:dots)ds.add(obj("abnormal",call(d,"isAbnormal"),"removed",call(d,"isRemoved")));
  Collection<?> live=(Collection<?>)call(sig,"edgeSet");for(Object e:edges)liveEdges.add(live.contains(e));
  return obj("dots",ds,"edges_live",liveEdges,"chord_dots",call(chord,"getDotsNumber"));
 }
 public static void main(String[] args)throws Exception{
  List<Object> cases=new ArrayList<>();
  String[] names={"upper_beats_grade","upper_taken_by_other_odd_head","both_extremes_taken_middle_ignored","equal_ordinate_insertion_order","negative_half_pitch_rounds_even","odd_current_head_takes_both"};
  for(int i=0;i<names.length;i++){
   Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig");
   for(Object n:new ArrayList<>((Collection<?>)call(sig,"vertexSet")))if(n.getClass().getSimpleName().equals("AugmentationDotInter"))call(n,"remove");
   List<Object> heads=new ArrayList<>();for(Object n:(Collection<?>)call(sig,"vertexSet"))if(n.getClass().getSimpleName().equals("HeadInter"))heads.add(n);
   heads.sort(Comparator.comparingInt(n->{try{return ((java.awt.Rectangle)call(n,"getBounds")).x;}catch(Exception e){throw new RuntimeException(e);}}));
   Object head=heads.get(0),other=heads.get(1),last=heads.get(2),chord=call(head,"getChord");
   call(head,"setPitch",i==5?1.0:0.0);call(other,"setPitch",i==4?-1.5:-1.0);call(last,"setPitch",1.0);
   List<Object> dots=new ArrayList<>(),edges=new ArrayList<>();int[] ys=i==2?new int[]{231,235,239}:i==3?new int[]{239,239}:new int[]{239,231};
   double[] grades=i==2?new double[]{.1,.9,.8}:new double[]{.9,.1};
   for(int j=0;j<ys.length;j++){Object d=dot(sheet,sig,head,303+j*6,ys[j]);dots.add(d);edges.add(edge(sig,d,head,"AugmentationRelation",grades[j]));}
   if(i==1||i==2||i==4)edges.add(edge(sig,dots.get(i==2?0:1),other,"AugmentationRelation",.7));
   if(i==2)edges.add(edge(sig,dots.get(2),last,"AugmentationRelation",.7));
   call(chord,"countDots");Object before=state(sig,chord,dots,edges),reducer=make("sig.SigReducer",sys,true);int mods=(Integer)invoke(reducer,"checkAugmentedHeads");Object after=state(sig,chord,dots,edges);
   List<Object> hs=new ArrayList<>();for(Object h:List.of(head,other,last))hs.add(obj("bounds",rect(call(h,"getBounds")),"pitch",call(h,"getPitch"),"integer_pitch",call(h,"getIntegerPitch")));
   cases.add(obj("name",names[i],"heads",hs,"chord_bounds",rect(call(chord,"getBounds")),"dot_bounds",dots.stream().map(d->{try{return rect(call(d,"getBounds"));}catch(Exception e){throw new RuntimeException(e);}}).toList(),"grades",grades,"additional_supports",i==2?List.of(List.of(0,1),List.of(2,2)):i==1||i==4?List.of(List.of(1,1)):List.of(),"before",before,"modifications",mods,"after",after,"repeat_modifications",invoke(reducer,"checkAugmentedHeads")));
  }
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("original_source","9e1e55cd2746037d059345881c53e6a6754bffbd","scope","declared temporary competing dots/grades/pitches on actual saved ordinary single HeadChords; original checkAugmentedHeads and removed callbacks; not image success","cases",cases));System.exit(0);
 }
}
