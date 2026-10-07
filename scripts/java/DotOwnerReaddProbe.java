// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;import java.util.*;import java.awt.Point;import java.awt.Rectangle;import com.fasterxml.jackson.databind.ObjectMapper;
public class DotOwnerReaddProbe extends DotPartOwnershipProbe {
 static Object readdState(Object sig,Object measure,List<Object> dots)throws Exception {
  List<Object> order=new ArrayList<>(),context=new ArrayList<>();for(Object d:(Collection<?>)call(measure,"getAugmentationDots"))if(dots.contains(d))order.add(dots.indexOf(d));for(Object d:dots)context.add(call(d,"getContextualGrade"));
  return obj("ownership",ownership(sig,measure,dots),"measure_order",order,"contextual_grades",context);
 }
 public static void main(String[] args)throws Exception {
  List<Object> cases=new ArrayList<>();for(boolean warm:List.of(false,true)){
   Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig"),dd=null,aug=null;
   for(Object e:(Collection<?>)call(sig,"edgeSet"))if(e.getClass().getSimpleName().equals("DoubleDotRelation")){dd=e;break;}Object first=call(sig,"getEdgeTarget",dd),second=call(sig,"getEdgeSource",dd);
   for(Object e:(Collection<?>)call(sig,"edgesOf",first))if(e.getClass().getSimpleName().equals("AugmentationRelation")){aug=e;break;}Object head=call(sig,"getEdgeTarget",aug),chord=call(head,"getChord"),measure=call(chord,"getMeasure");List<Object> dots=List.of(first,second),steps=new ArrayList<>();
   if(warm){call(first,"getPart");call(second,"getPart");}
   Object before=readdState(sig,measure,dots);call(first,"remove");call(second,"remove");steps.add(obj("action","remove both","state",readdState(sig,measure,dots)));
   call(sig,"addVertex",second);steps.add(obj("action","readd second","state",readdState(sig,measure,dots)));
   call(sig,"addVertex",first);steps.add(obj("action","readd first","state",readdState(sig,measure,dots)));
   call(sig,"addEdge",first,head,aug);call(sig,"addEdge",second,first,dd);steps.add(obj("action","restore support","state",readdState(sig,measure,dots)));
   invoke(make("sheet.rhythm.PageRhythm",call(sys,"getPage")),"populateFRATs");steps.add(obj("action","populateFRATs","state",readdState(sig,measure,dots)));
   cases.add(obj("warm",warm,"before",before,"steps",steps,"dot_grades",List.of(call(first,"getGrade"),call(second,"getGrade"))));
  }
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig");List<?> stacks=(List<?>)call(sys,"getStacks");Object firstStack=stacks.get(0),head=null;
  for(Object n:(Collection<?>)call(sig,"vertexSet"))if(n.getClass().getSimpleName().equals("HeadInter")){head=n;break;}
  int left=(Integer)call(firstStack,"getLeft"),right=(Integer)call(firstStack,"getRight"),y=((Point)call(head,"getCenter")).y;List<Object> boundary=new ArrayList<>(),stackFacts=new ArrayList<>();
  for(Object stack:stacks)stackFacts.add(obj("left",call(stack,"getLeft"),"right",call(stack,"getRight")));
  for(int x:new int[]{left,right,right+1}){
   Object dot=make("sig.inter.AugmentationDotInter",call(head,"getGlyph"),.8);call(dot,"setBounds",new Rectangle(x-2,y-2,5,4));call(dot,"setStaff",call(head,"getStaff"));Object selected=call(sys,"getStackAt",new Point(x,y));List<Boolean> filtered=new ArrayList<>();for(Object stack:stacks)filtered.add(((Collection<?>)call(stack,"filter",List.of(dot))).contains(dot));call(sig,"addVertex",dot);List<Boolean> members=new ArrayList<>();for(Object stack:stacks){Object measure=call(stack,"getMeasureAt",call(head,"getStaff"));members.add(((Collection<?>)call(measure,"getAugmentationDots")).contains(dot));}
   boundary.add(obj("center_x",x,"getStackAt_index",selected==null?null:stacks.indexOf(selected)+1,"stack_filter",filtered,"added_membership",members,"cached_part_id",cached(dot)));call(dot,"remove");
  }
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("original_source","9e1e55cd2746037d059345881c53e6a6754bffbd","scope","actual saved chain cold/warm removal and same-Inter re-addition; declared scalar stack boundaries using reused glyph and explicit bounds, not image recognition/export","cases",cases,"stack_bounds",stackFacts,"boundary",boundary));System.exit(0);
 }
}
