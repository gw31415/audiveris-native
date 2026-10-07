// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;import java.util.*;import com.fasterxml.jackson.databind.ObjectMapper;
public class WeakDotCascadeProbe extends DotCallbackOrderProbe {
 public static void main(String[] args)throws Exception {
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig");
  for(Object n:new ArrayList<>((Collection<?>)call(sig,"vertexSet")))if(n.getClass().getSimpleName().equals("AugmentationDotInter"))call(n,"remove");
  List<Object> owners=new ArrayList<>();for(Object n:(Collection<?>)call(sig,"vertexSet"))if(n.getClass().getSimpleName().equals("HeadChordInter"))owners.add(n);
  owners.sort(Comparator.comparingInt(n->{try{return ((java.awt.Rectangle)call(n,"getBounds")).x;}catch(Exception e){throw new RuntimeException(e);}}));
  Object chord=owners.get(0),old=owners.get(1),h0=((List<?>)call(chord,"getNotes")).get(0),h1=((List<?>)call(old,"getNotes")).get(0);call(old,"removeMember",h1);call(chord,"addMember",h1);
  List<Object> notes=List.of(h0,h1),chords=List.of(chord),dots=new ArrayList<>(),edges=new ArrayList<>(),facts=new ArrayList<>();
  for(int i=0;i<2;i++){Object h=notes.get(i);java.awt.Rectangle b=(java.awt.Rectangle)call(h,"getBounds");Object d=dot(sheet,sig,h,b.x+b.width+12,b.y-5);call(d,"setGrade",i==0?.1:.8);call(call(chord,"getMeasure"),"addInter",d);dots.add(d);edges.add(edge(sig,d,h,"AugmentationRelation",.7));facts.add(obj("kind","HeadInter","shape",call(h,"getShape").toString(),"bounds",rect(call(h,"getBounds")),"grade",call(h,"getGrade")));}
  call(chord,"countDots");List<Object> contextual=new ArrayList<>();for(Object d:dots)contextual.add(call(sig,"computeContextualGrade",d));if((Double)contextual.get(0)>=.5||(Double)contextual.get(1)<.5)throw new IllegalStateException("weak/strong boundary not established");
  Object before=state(sig,notes,chords,dots,edges);List<Object> trace=new ArrayList<>();watch(sig,notes,chords,dots,edges,trace);call(sig,"deleteInters",List.of(dots.get(0)));Object after=state(sig,notes,chords,dots,edges);
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("original_source","9e1e55cd2746037d059345881c53e6a6754bffbd","scope","selected original weak-dot computeContextualGrade/deleteInters on declared temporary unshared dots and explicit transfer of actual saved member; callback removes strong automatic dot; not full SIG purge or image success","notes",facts,"chord_bounds",rect(call(chord,"getBounds")),"member_order",((List<?>)call(chord,"getNotes")).stream().map(n->notes.indexOf(n)).toList(),"dot_bounds",dots.stream().map(d->{try{return rect(call(d,"getBounds"));}catch(Exception e){throw new RuntimeException(e);}}).toList(),"dot_grades",List.of(.1,.8),"dot_contextual_grades",contextual,"before",before,"trace",trace,"after",after));System.exit(0);
 }
}
