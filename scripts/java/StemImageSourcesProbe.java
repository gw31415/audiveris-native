// SPDX-License-Identifier: AGPL-3.0-or-later
// Original inspectStems before linking/cleaner; actual image-derived CLinker seed inputs.
import org.audiveris.omr.sheet.Book;import java.util.*;import java.nio.file.*;import java.lang.reflect.*;import java.awt.geom.*;
public class StemImageSourcesProbe extends StemPartsLookupProbe {
 private final Object book;public StemImageSourcesProbe(Book b,SortedSet<Integer> ids){super(b,ids);book=b;}
 static Object inspect(Object sys,Object sheet)throws Exception {
  Object r=Class.forName("org.audiveris.omr.sheet.stem.StemsRetriever").getConstructor(Class.forName("org.audiveris.omr.sheet.SystemInfo")).newInstance(sys);
  Constructor<?> ctor=Class.forName("org.audiveris.omr.sheet.stem.StemsRetriever$Parameters").getDeclaredConstructor(Class.forName("org.audiveris.omr.sheet.SystemInfo"),Class.forName("org.audiveris.omr.sheet.Scale"));ctor.setAccessible(true);
  set(r,"params",ctor.newInstance(sys,call(sheet,"getScale")));set(r,"watch",Class.forName("org.audiveris.omr.util.StopWatch").getConstructor(String.class).newInstance("original image inputs observation"));
  call(r,"inspectStems");return r;
 }
 static String line(Object x){Line2D l=(Line2D)x;return "["+l.getX1()+","+l.getY1()+","+l.getX2()+","+l.getY2()+"]";}
 static String glyphs(Collection<?> x)throws Exception {List<String>r=new ArrayList<>();for(Object g:x)r.add(physical(g));return "["+String.join(",",r)+"]";}
 static String control(Object c,Object hl,Object p,String name,int[][]boxes,int[]stump,double dx,int min)throws Exception {
  Collection<Object> neighbors=new LinkedHashSet<>();List<String>input=new ArrayList<>();for(int[]b:boxes){Object g=glyph(b[0],b[1],b[2],b[3],"VERTICAL");neighbors.add(g);input.add(physical(g));}
  set(hl,"neighborSeeds",neighbors);Object s=stump==null?null:glyph(stump[0],stump[1],stump[2],stump[3],"VERTICAL");set(c,"stump",s);
  set(c,"luArea",new Area(new Rectangle2D.Double(94,100,20,40)));set(c,"theoLine",new Line2D.Double(100,100,100,140));set(c,"yRange",new java.awt.Rectangle(0,100,0,40));set(p,"minSeedContrib",min);set(p,"maxLineSeedDx",dx);
  return "{\"name\":\""+name+"\",\"candidates\":["+String.join(",",input)+"],\"stump\":"+(s==null?"null":physical(s))+",\"polygon\":[[94,100],[114,100],[114,140],[94,140]],\"theoretical_line\":[100,100,100,140],\"y_range\":[0,100,0,40],\"min_seed_contribution\":"+min+",\"max_line_seed_dx\":"+dx+",\"selected\":"+glyphs((Collection<?>)call(c,"retrieveSeeds"))+"}";
 }
 public void process(){try{
  Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),firstCorner=null,firstHeadLinker=null,firstParams=null;List<String>systems=new ArrayList<>();int count=0,seedCount=0,filteredCount=0;
  for(Object sys:(List<?>)call(sheet,"getSystems")){
   Object retr=inspect(sys,sheet),p=get(retr,"params");List<String>cases=new ArrayList<>();
   for(Object head:(List<?>)get(retr,"systemHeads")){Object hl=call(head,"getLinker");for(String side:new String[]{"LEFT","RIGHT"})for(String vert:new String[]{"TOP","BOTTOM"}){
    Object c=call(hl,"getCornerLinker",en("util.HorizontalSide",side),en("util.VerticalSide",vert));if(firstCorner==null){firstCorner=c;firstHeadLinker=hl;firstParams=p;}Collection<?> seeds=(Collection<?>)call(c,"retrieveSeeds"),filtered=(Collection<?>)get(c,"seeds");filteredCount+=filtered.size();Object stump=call(c,"getStump");seedCount+=seeds.size();count++;
    cases.add("{\"head_bounds\":"+rect(call(head,"getBounds"))+",\"shape\":\""+call(head,"getShape")+"\",\"side\":\""+side+"\",\"vertical\":\""+vert+"\",\"area_path\":"+path(get(c,"luArea"))+",\"theoretical_line\":"+line(get(c,"theoLine"))+",\"y_range\":"+rect(get(c,"yRange"))+",\"stump\":"+(stump==null?"null":physical(stump))+",\"selected_seeds\":"+glyphs(seeds)+",\"aligned_seeds\":"+glyphs(filtered)+"}");
   }}
   systems.add("{\"system\":"+call(sys,"getId")+",\"min_seed_contribution\":"+get(p,"minSeedContrib")+",\"max_line_seed_dx\":"+get(p,"maxLineSeedDx")+",\"max_alignment_dx\":"+get(p,"maxStemAlignmentDx")+",\"max_alignment_dy\":"+get(p,"maxStemAlignmentDy")+",\"cases\":["+String.join(",",cases)+"]}");
  }
  List<String>controls=new ArrayList<>();int[][]base={{100,100,1,10},{99,106,1,8},{100,115,1,6}};
  controls.add(control(firstCorner,firstHeadLinker,firstParams,"empty",new int[][]{},null,1.65,6));
  controls.add(control(firstCorner,firstHeadLinker,firstParams,"greedy-overlap",base,null,1.65,6));
  controls.add(control(firstCorner,firstHeadLinker,firstParams,"stump-overlap-and-touch",base,new int[]{100,100,1,6},1.65,6));
  for(int h:new int[]{5,6,7})controls.add(control(firstCorner,firstHeadLinker,firstParams,"contribution-"+h,new int[][]{{100,110,1,h}},null,1.65,6));
  for(double dx:new double[]{Math.nextDown(1.),1.,Math.nextUp(1.)})controls.add(control(firstCorner,firstHeadLinker,firstParams,"distance-"+dx,new int[][]{{101,110,1,6}},null,dx,6));
  controls.add(control(firstCorner,firstHeadLinker,firstParams,"area-touch",new int[][]{{114,110,1,6}},null,20,6));
  controls.add(control(firstCorner,firstHeadLinker,firstParams,"area-overlap",new int[][]{{113,110,1,6}},null,20,6));
  controls.add(control(firstCorner,firstHeadLinker,firstParams,"clipped-contribution",new int[][]{{100,95,1,10},{100,98,1,8}},null,1.65,6));
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"native_outputs_used\":false,\"original_phase\":\"inspectStems_before_linkStems_and_finalizeStems\",\"corners\":"+count+",\"aligned_seed_occurrences\":"+filteredCount+",\"selected_seed_occurrences\":"+seedCount+",\"systems\":["+String.join(",",systems)+"],\"seed_source_controls\":["+String.join(",",controls)+"]}");
 }catch(Exception e){throw new RuntimeException(e);}}
}
