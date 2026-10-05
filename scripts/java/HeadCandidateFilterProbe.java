// SPDX-License-Identifier: AGPL-3.0-or-later
// Original aggregateMatches/filterSeedConflicts on live range candidates and declared controls.
import org.audiveris.omr.sheet.Book;import java.util.*;import java.nio.file.*;import java.awt.Rectangle;
public class HeadCandidateFilterProbe extends HeadRangeCatalogProbe {
 List<Object> matches; IdentityHashMap<Object,Integer> positions=new IdentityHashMap<>();
 public HeadCandidateFilterProbe(Book b,SortedSet<Integer> ids){super(b,ids);}
 String match(Object table,Object catalog,int x,int y,int[]xo,int[]yo,String an,String sh)throws Exception{
  String record=super.match(table,catalog,x,y,xo,yo,an,sh);
  if(!record.equals("null")&&lastEligible&&matches!=null){Object a=Class.forName("org.audiveris.omr.image.Anchored$Anchor").getField(an).get(null),shape=Class.forName("org.audiveris.omr.glyph.Shape").getField(token(record,"shape")).get(null),loc=make("image.PixelDistance",Integer.parseInt(token(record,"x")),Integer.parseInt(token(record,"y")),Double.parseDouble(token(record,"score")));Object h=call(get(activeScanner,"this$0"),"createInter",loc,a,shape,lastStaff,((Number)get(activeScanner,"pitch")).doubleValue());positions.put(h,matches.size());matches.add(h);}
  return record;
 }
 String indexes(List<?> list){List<String> out=new ArrayList<>();for(Object h:list)out.add(positions.get(h).toString());return "["+String.join(",",out)+"]";}
 String scan(Object b,Object adapter,Object second,Object table,Object catalog,int dir,int pitch,String identity)throws Exception{
  matches=new ArrayList<>();positions.clear();String raw=super.scan(b,adapter,second,table,catalog,dir,pitch,identity);
  List<?> aggregated=(List<?>)call(b,"aggregateMatches",new ArrayList<>(matches));List<?> filtered=(List<?>)call(b,"filterSeedConflicts",aggregated,get(activeScanner,"competitors"));
  return raw.substring(0,raw.length()-1)+",\"candidate_filter\":{\"aggregate_indices\":"+indexes(aggregated)+",\"filtered_indices\":"+indexes(filtered)+",\"max_template_dx\":"+get(get(b,"params"),"maxTemplateDx")+",\"seed_competitors\":[]}}";
 }
 Object head(int[]box,double grade,String shape)throws Exception{return make("sig.inter.HeadInter",new Rectangle(box[0],box[1],box[2],box[3]),Class.forName("org.audiveris.omr.glyph.Shape").getField(shape).get(null),grade,lastStaff,0.);}
 String row(Object head,int index)throws Exception{return "{\"input_index\":"+index+",\"kind\":\"HeadInter\",\"shape\":\""+call(head,"getShape")+"\",\"bounds\":"+rect(call(head,"getBounds"))+",\"grade_bits\":"+Double.doubleToRawLongBits((Double)call(head,"getGrade"))+"}";}
 void controls(Path output)throws Exception{
  matches=null;Object b=get(activeScanner,"this$0");List<String> cases=new ArrayList<>();
  int[][][] boxes={ {},{{0,0,10,8}},{{0,0,10,8},{4,40,10,8},{8,0,10,8}},{{0,0,10,8},{4,0,10,8},{8,0,10,8}},{{0,0,10,8},{0,100,10,8}},{{0,0,10,8},{5,0,11,8}},{{0,0,10,8},{4,0,11,8}},{{0,0,10,8},{0,0,10,8},{20,0,10,8}}};
  for(int n=0;n<boxes.length;n++){List<Object> hs=new ArrayList<>();positions.clear();List<String> inputs=new ArrayList<>();for(int i=0;i<boxes[n].length;i++){double grade=n==3&&i==1?.8:n==4||n==7?.7:.8-i*.1;Object h=head(boxes[n][i],grade,i%2==0?"NOTEHEAD_BLACK":"NOTEHEAD_VOID");positions.put(h,i);hs.add(h);inputs.add(row(h,i));}List<?> result=(List<?>)call(b,"aggregateMatches",new ArrayList<>(hs));cases.add("{\"name\":\"aggregate-"+n+"\",\"heads\":["+String.join(",",inputs)+"],\"competitors\":[],\"aggregate_indices\":"+indexes(result)+",\"filtered_indices\":"+indexes(result)+"}");}
  for(int shift:new int[]{9,10,11,12})for(double grade:new double[]{Math.nextDown(.7*.9),.7*.9,Math.nextUp(.7*.9)}){
   Object h=head(new int[]{0,0,11,1},.7,"WHOLE_NOTE"),c=head(new int[]{shift,0,11,1},grade,"NOTEHEAD_BLACK");positions.clear();positions.put(h,0);List<?> result=(List<?>)call(b,"filterSeedConflicts",new ArrayList<>(List.of(h)),new ArrayList<>(List.of(c)));
   cases.add("{\"name\":\"conflict-"+shift+"-"+Double.doubleToRawLongBits(grade)+"\",\"heads\":["+row(h,0)+"],\"competitors\":["+row(c,0)+"],\"aggregate_indices\":[0],\"filtered_indices\":"+indexes(result)+"}");
  }
  {
   List<Object> hs=new ArrayList<>(),cs=new ArrayList<>();List<String> inputs=new ArrayList<>(),comps=new ArrayList<>();positions.clear();
   for(int i=0;i<3;i++){Object h=head(new int[]{i*20,0,11,1},.7,"NOTEHEAD_VOID");positions.put(h,i);hs.add(h);inputs.add(row(h,i));}
   for(int i=0;i<3;i++){Object c=head(new int[]{i==0?9:29,i==2?20:0,11,1},i==0?.62:.64,"NOTEHEAD_BLACK");cs.add(c);comps.add(row(c,i));}
   List<?> result=(List<?>)call(b,"filterSeedConflicts",hs,cs);
   cases.add("{\"name\":\"parallel-competitors\",\"heads\":["+String.join(",",inputs)+"],\"competitors\":["+String.join(",",comps)+"],\"aggregate_indices\":[0,1,2],\"filtered_indices\":"+indexes(result)+"}");
  }
  Files.writeString(output.resolveSibling("filter-controls.json"),"{\"native_outputs_used\":false,\"max_template_dx\":"+get(get(b,"params"),"maxTemplateDx")+",\"grade_margin\":0.1,\"min_iou_heads\":0.1,\"cases\":["+String.join(",",cases)+"],\"scope\":\"Declared original unregistered HeadInter controls; stable grade ties, fixed representative, y-independent aggregation, half centers, empty input, IoU equality/touching/separation and adjacent grade bits.\"}\n");
 }
}
