// SPDX-License-Identifier: AGPL-3.0-or-later
// Actual original builder items; supplied-item gap/length controls, not native page selection.
import org.audiveris.omr.sheet.Book;
import java.util.*;import java.nio.file.*;import java.awt.geom.*;
public class StemLengthMapProbe extends StemPartsLookupProbe {
 private final Object book;public StemLengthMapProbe(Book b,SortedSet<Integer> ids){super(b,ids);book=b;}
 static String item(Object i)throws Exception{
  if(i==null)return "null";
  Line2D line=(Line2D)get(i,"line");String head="null";
  if(Class.forName("org.audiveris.omr.sheet.stem.StemItem$LinkerItem").isInstance(i)){
   Object linker=get(i,"linker");if(linker.getClass().getSimpleName().equals("CLinker"))head=rect(call(call(linker,"getSource"),"getBounds"));
  }
  return "{\"kind\":\""+i.getClass().getSimpleName()+"\",\"gap\":"+i.getClass().getSimpleName().equals("GapItem")+",\"contrib\":"+get(i,"contrib")+",\"line\":"+(line==null?"null":Arrays.toString(new double[]{line.getX1(),line.getY1(),line.getX2(),line.getY2()}))+",\"line_bounds\":"+(line==null?"null":rect(line.getBounds()))+",\"head_bounds\":"+head+"}";
 }
 static String map(Map<Integer,Integer> m){List<String> rows=new ArrayList<>();for(var e:m.entrySet())rows.add("\""+e.getKey()+"\":"+e.getValue());return "{"+String.join(",",rows)+"}";}
 static String lengths(Object sb,int size)throws Exception{List<String>r=new ArrayList<>();for(int i=-1;i<size;i++)r.add(""+call(sb,"getLengthAt",i));return "["+String.join(",",r)+"]";}
 static String record(Object sb,String mode,int profile,Integer gi,Integer gc,Map<Integer,Integer> initial)throws Exception{
  @SuppressWarnings("unchecked") Map<Integer,Integer> lm=(Map<Integer,Integer>)get(sb,"lengthMap");lm.clear();lm.putAll(initial);call(sb,"retrieveLengths",profile);
  return "{\"mode\":\""+mode+"\",\"max_profile\":"+profile+",\"gap_index\":"+gi+",\"gap_contrib\":"+gc+",\"initial_lengths\":"+map(initial)+",\"lengths\":"+map(lm)+"}";
 }
 public void process(){try{
  Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0);
  Object retr=Class.forName("org.audiveris.omr.sheet.stem.StemsRetriever").getConstructor(Class.forName("org.audiveris.omr.sheet.SystemInfo")).newInstance(sys);call(retr,"process");
  Set<Object>seen=Collections.newSetFromMap(new IdentityHashMap<>());List<Object>builders=new ArrayList<>();
  for(Object head:(List<?>)get(retr,"systemHeads")){Object linker=call(head,"getLinker");if(linker==null)continue;for(String side:new String[]{"LEFT","RIGHT"})for(String vert:new String[]{"TOP","BOTTOM"}){Object c=call(linker,"getCornerLinker",en("util.HorizontalSide",side),en("util.VerticalSide",vert)),sb=get(c,"sb");if(sb!=null&&seen.add(sb))builders.add(sb);}}
  for(Object beam:(List<?>)get(retr,"systemBeams")){Object linker=call(beam,"getLinker");if(linker==null)continue;for(Object side:(List<?>)get(linker,"allBLinkers"))for(Object half:(Collection<?>)call(side,"getHalfLinkers")){Object sb=get(half,"sb");if(sb!=null&&seen.add(sb))builders.add(sb);}}
  @SuppressWarnings("unchecked") Map<Integer,Integer> gapMap=(Map<Integer,Integer>)call(retr,"getGapMap");List<String>groups=new ArrayList<>();Set<Integer>directions=new HashSet<>();
  for(Object sb:builders){List<?>items=(List<?>)get(sb,"items");Line2D line=(Line2D)get(sb,"theoLine");@SuppressWarnings("unchecked") Map<Integer,Integer> lm=(Map<Integer,Integer>)get(sb,"lengthMap");Map<Integer,Integer>saved=new TreeMap<>(lm);List<String>ir=new ArrayList<>(),rows=new ArrayList<>(),starts=new ArrayList<>();int firstGap=-1;for(int i=0;i<items.size();i++){ir.add(item(items.get(i)));if(firstGap<0&&items.get(i).getClass().getSimpleName().equals("GapItem"))firstGap=i;}
   String prefixes=lengths(sb,items.size());int dir=(Integer)get(sb,"yDir");
   try{
    for(int profile:gapMap.keySet()){
     rows.add(record(sb,"actual",profile,null,null,Map.of()));rows.add(record(sb,"preserve-first-profile",profile,null,null,Map.of(gapMap.keySet().iterator().next(),77)));
     try{set(sb,"items",new ArrayList<>());rows.add(record(sb,"empty",profile,null,null,Map.of()));}finally{set(sb,"items",items);}
    }
    if(firstGap>=0){Object gap=items.get(firstGap);int orig=(Integer)get(gap,"contrib");try{for(var e:gapMap.entrySet())for(int delta:new int[]{-1,0,1}){
     int v=e.getValue()+delta;set(gap,"contrib",v);for(int profile:gapMap.keySet())rows.add(record(sb,"threshold-"+e.getKey()+"-"+delta,profile,firstGap,v,Map.of()));
    }}finally{set(gap,"contrib",orig);}}
    if(directions.add(dir)){for(double y:new double[]{-20.75,-0.5,0.5,20.75}){try{set(sb,"theoLine",new Line2D.Double(line.getX1(),y,line.getX2(),line.getY2()));starts.add("{\"start_y\":"+y+",\"prefix_lengths\":"+lengths(sb,items.size())+"}");}finally{set(sb,"theoLine",line);}}}
   }finally{lm.clear();lm.putAll(saved);}
   groups.add("{\"start_kind\":\""+get(sb,"startLinker").getClass().getSimpleName()+"\",\"direction\":"+dir+",\"start_y\":"+line.getY1()+",\"items\":["+String.join(",",ir)+"],\"prefix_lengths\":"+prefixes+",\"observed_length_map\":"+map(saved)+",\"start_y_controls\":["+String.join(",",starts)+"],\"cases\":["+String.join(",",rows)+"]}");
  }
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"native_outputs_used\":false,\"scope\":\"612 actual first-system builders; original items and Line2D bounds supplied. Direct getLengthAt/retrieveLengths with declared max profiles, first gap threshold controls, sticky initial lengths, empty items and selected signed fractional reference starts. Native automatic items/parts/page selection and epilog remain incomplete\",\"gap_map\":"+map(gapMap)+",\"groups\":["+String.join(",",groups)+"]}");
 }catch(Exception e){throw new RuntimeException(e);}}
}
