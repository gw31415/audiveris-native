// SPDX-License-Identifier: AGPL-3.0-or-later
// Original item constructors, suffix sort/gap events and composed length maps.
import org.audiveris.omr.sheet.Book;
import java.util.*;import java.nio.file.*;import java.awt.geom.*;
public class StemItemEventsProbe extends StemLengthMapProbe {
 private final Object book;public StemItemEventsProbe(Book b,SortedSet<Integer> ids){super(b,ids);book=b;}
 static String source(Object i,int token)throws Exception{
  String kind=i.getClass().getSimpleName(),ref="null",height="null",head="null";Object glyph=get(i,"glyph");
  if(!kind.equals("GlyphItem")){Object linker=get(i,"linker");Point2D r=(Point2D)call(linker,"getReferencePoint");ref=Arrays.toString(new double[]{r.getX(),r.getY()});if(linker.getClass().getSimpleName().equals("BLinker"))height=""+call(call(linker,"getSource"),"getHeight");if(linker.getClass().getSimpleName().equals("CLinker"))head=rect(call(call(linker,"getSource"),"getBounds"));}
  return "{\"token\":"+token+",\"kind\":\""+kind+"\",\"glyph\":"+(glyph==null?"null":physical(glyph))+",\"reference\":"+ref+",\"beam_height\":"+height+",\"head_bounds\":"+head+"}";
 }
 static Object rebuild(Object sb,Object i)throws Exception{
  String kind=i.getClass().getSimpleName();Object g=get(i,"glyph");int c=g==null?0:(Integer)call(sb,"getContrib",call(g,"getBounds"));
  if(kind.equals("GlyphItem"))return i.getClass().getConstructor(Class.forName("org.audiveris.omr.glyph.Glyph"),int.class).newInstance(g,c);
  Object linker=get(i,"linker");if(kind.equals("HalfLinkerItem"))return i.getClass().getConstructor(Class.forName("org.audiveris.omr.sheet.stem.StemHalfLinker"),int.class).newInstance(linker,c);
  return i.getClass().getConstructor(Class.forName("org.audiveris.omr.sheet.stem.StemLinker")).newInstance(linker);
 }
 static String prepared(Object i,Integer token)throws Exception{
  String s=item(i);String reference="null";if(i.getClass().getSimpleName().equals("HalfLinkerItem")){Point2D p=(Point2D)call(get(i,"linker"),"getReferencePoint");reference=Arrays.toString(new double[]{p.getX(),p.getY()});}
  return s.substring(0,s.length()-1)+",\"token\":"+token+",\"reference\":"+reference+"}";
 }
 static String record(Object sb,String mode,int profile,List<Object>input,List<String>sources)throws Exception{
  List<?> old=(List<?>)get(sb,"items");@SuppressWarnings("unchecked") Map<Integer,Integer>lm=(Map<Integer,Integer>)get(sb,"lengthMap");Map<Integer,Integer> saved=new TreeMap<>(lm);IdentityHashMap<Object,Integer>tags=new IdentityHashMap<>();for(int j=0;j<input.size();j++)tags.putIfAbsent(input.get(j),j+1);
  List<String>before=new ArrayList<>();for(Object i:input)before.add(prepared(i,tags.get(i)));
  try{List<Object> list=new ArrayList<>(input);set(sb,"items",list);if(list.size()>1)call(sb,"sortItems",list.subList(1,list.size()));List<String>sorted=new ArrayList<>();for(Object i:list)sorted.add(""+tags.get(i));call(sb,"insertGapEvents",profile);List<?>events=(List<?>)get(sb,"items");List<String>out=new ArrayList<>();for(Object i:events)out.add(prepared(i,tags.get(i)));lm.clear();call(sb,"retrieveLengths",4);
   return "{\"mode\":\""+mode+"\",\"stem_profile\":"+profile+",\"max_profile\":4,\"sources\":["+String.join(",",sources)+"],\"prepared_input\":["+String.join(",",before)+"],\"sorted_tokens\":["+String.join(",",sorted)+"],\"items\":["+String.join(",",out)+"],\"prefix_lengths\":"+lengths(sb,events.size())+",\"lengths\":"+map(lm)+"}";
  }finally{set(sb,"items",old);lm.clear();lm.putAll(saved);}
 }
 public void process(){try{
  Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),retr=Class.forName("org.audiveris.omr.sheet.stem.StemsRetriever").getConstructor(Class.forName("org.audiveris.omr.sheet.SystemInfo")).newInstance(sys);call(retr,"process");
  Set<Object>seen=Collections.newSetFromMap(new IdentityHashMap<>());List<Object>builders=new ArrayList<>();for(Object head:(List<?>)get(retr,"systemHeads")){Object linker=call(head,"getLinker");if(linker==null)continue;for(String side:new String[]{"LEFT","RIGHT"})for(String vert:new String[]{"TOP","BOTTOM"}){Object c=call(linker,"getCornerLinker",en("util.HorizontalSide",side),en("util.VerticalSide",vert)),sb=get(c,"sb");if(sb!=null&&seen.add(sb))builders.add(sb);}}for(Object beam:(List<?>)get(retr,"systemBeams")){Object linker=call(beam,"getLinker");if(linker==null)continue;for(Object side:(List<?>)get(linker,"allBLinkers"))for(Object half:(Collection<?>)call(side,"getHalfLinkers")){Object sb=get(half,"sb");if(sb!=null&&seen.add(sb))builders.add(sb);}}
  List<String>groups=new ArrayList<>();Set<Integer> boundaryDirections=new HashSet<>();@SuppressWarnings("unchecked") Map<Integer,Integer>gaps=(Map<Integer,Integer>)call(retr,"getGapMap");
  for(Object sb:builders){List<Object>base=new ArrayList<>();for(Object i:(List<?>)get(sb,"items"))if(!i.getClass().getSimpleName().equals("GapItem"))base.add(rebuild(sb,i));List<String>rows=new ArrayList<>();
   for(int profile:gaps.keySet()){List<String>sources=new ArrayList<>();for(int j=0;j<base.size();j++)sources.add(source(base.get(j),j+1));rows.add(record(sb,"actual",profile,base,sources));}
   if(base.size()>1){for(String mode:new String[]{"reversed-suffix","duplicate-first-suffix"}){List<Object>input=new ArrayList<>(base);if(mode.equals("reversed-suffix"))Collections.reverse(input.subList(1,input.size()));else input.add(base.get(1));IdentityHashMap<Object,Integer>tags=new IdentityHashMap<>();List<String>sources=new ArrayList<>();for(int j=0;j<input.size();j++){tags.putIfAbsent(input.get(j),j+1);sources.add(source(input.get(j),tags.get(input.get(j))));}rows.add(record(sb,mode,1,input,sources));}}
   rows.add(record(sb,"empty",1,new ArrayList<>(),new ArrayList<>()));
   List<String>boundary=new ArrayList<>();int dir=(Integer)get(sb,"yDir");if(boundaryDirections.add(dir)&&base.size()>1){Object a=base.get(0),b=base.get(1);Object la=get(a,"line"),lb=get(b,"line");try{for(double d:new double[]{Math.nextDown(0.01),0.01,Math.nextUp(0.01),Math.nextDown(3.),3.,Math.nextUp(3.),Math.nextDown(44.),44.,Math.nextUp(44.)}){
     set(a,"line",new Line2D.Double(0.,0.,0.,0.));set(b,"line",dir>0?new Line2D.Double(0.,d,0.,d+1.):new Line2D.Double(0.,-d-1.,0.,-d));boundary.add(record(sb,"declared-line-boundary",d>40?4:1,new ArrayList<>(List.of(a,b)),new ArrayList<>()));
    }}finally{set(a,"line",la);set(b,"line",lb);}}
   groups.add("{\"direction\":"+dir+",\"start_y\":"+((Line2D)get(sb,"theoLine")).getY1()+",\"y_range\":"+rect(get(sb,"yRange"))+",\"cases\":["+String.join(",",rows)+"],\"declared_line_boundaries\":["+String.join(",",boundary)+"]}");
  }
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"native_outputs_used\":false,\"scope\":\"Original constructors on actual prefiltered surviving item sources from612 Laura builders, suffix TimSort, profile gap events and retrieveLengths. Declared physical glyph/linker sources, not native image automatic target/filter selection or complete page cache/SIG/epilog. Boundary controls override original item lines explicitly\",\"gap_map\":"+map(gaps)+",\"groups\":["+String.join(",",groups)+"]}");
 }catch(Exception e){throw new RuntimeException(e);}}
}
