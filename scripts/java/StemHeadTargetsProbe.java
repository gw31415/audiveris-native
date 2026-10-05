// SPDX-License-Identifier: AGPL-3.0-or-later
// Original neighborhood supplied; actual lookupBeamGroups and computeTargetPoint.
import org.audiveris.omr.sheet.Book;
import java.util.*;import java.nio.file.*;import java.awt.geom.*;
public class StemHeadTargetsProbe extends StemPartsLookupProbe {
 private final Object book;
 public StemHeadTargetsProbe(Book b,SortedSet<Integer> ids){super(b,ids);book=b;}
 static String pt(Point2D p){return "["+p.getX()+","+p.getY()+"]";}
 static String ln(Line2D p){return "["+p.getX1()+","+p.getY1()+","+p.getX2()+","+p.getY2()+"]";}
 static String tokens(Collection<?> values,Map<Object,Integer> map){List<String> r=new ArrayList<>();for(Object v:values){Integer id=map.get(v);if(id==null)throw new IllegalStateException("unknown beam/group");r.add(id.toString());}return "["+String.join(",",r)+"]";}
 static Object sc(String cls,String name,Object... args)throws Exception {for(java.lang.reflect.Method m:Class.forName("org.audiveris.omr."+cls).getDeclaredMethods())if(m.getName().equals(name)&&m.getParameterCount()==args.length){m.setAccessible(true);return m.invoke(null,args);}throw new NoSuchMethodException(name);}
 public void process(){try{
 Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0);
 Object retr=Class.forName("org.audiveris.omr.sheet.stem.StemsRetriever").getConstructor(Class.forName("org.audiveris.omr.sheet.SystemInfo")).newInstance(sys);call(retr,"process");
 List<Object> beams=new ArrayList<>((List<?>)get(retr,"systemBeams"));Map<Object,Integer> bid=new IdentityHashMap<>(),gid=new IdentityHashMap<>();List<Object> groups=new ArrayList<>();
 for(int i=0;i<beams.size();i++){bid.put(beams.get(i),i+1);Object g=call(beams.get(i),"getGroup");if(g!=null&&!gid.containsKey(g)){gid.put(g,gid.size()+1);groups.add(g);}}
 List<String> catalog=new ArrayList<>(),groupRows=new ArrayList<>();for(Object b:beams){Object g=call(b,"getGroup");catalog.add("{\"id\":"+bid.get(b)+",\"group\":"+(g==null?"null":gid.get(g))+",\"median\":"+ln((Line2D)call(b,"getMedian"))+",\"top\":"+ln((Line2D)call(b,"getBorder",en("util.VerticalSide","TOP")))+",\"bottom\":"+ln((Line2D)call(b,"getBorder",en("util.VerticalSide","BOTTOM")))+",\"height\":"+call(b,"getHeight")+"}");}
 for(Object g:groups)groupRows.add("{\"id\":"+gid.get(g)+",\"members\":"+tokens((List<?>)call(g,"getMembers"),bid)+"}");
 List<String> rows=new ArrayList<>(),boundary=new ArrayList<>();int count=0;
 for(Object head:(List<?>)get(retr,"systemHeads")){Object hl=call(head,"getLinker");if(hl==null)continue;
 for(String side:new String[]{"LEFT","RIGHT"})for(String vert:new String[]{"TOP","BOTTOM"}){Object c=call(hl,"getCornerLinker",en("util.HorizontalSide",side),en("util.VerticalSide",vert));if(get(c,"sb")==null)continue;count++;
 Object params=get(hl,"params"),part=call(call(head,"getStaff"),"getPart"),sl=get(c,"this$1");Point2D ref=(Point2D)get(c,"refPt");int dir=(Integer)get(c,"yDir"),min=(Integer)get(params,"minBeamHeadDy");List<Object> neighbors=new ArrayList<>((List<?>)get(hl,"neighborBeams"));
 Object oldTheory=get(c,"theoLine"),oldArea=get(c,"luArea"),oldTarget=get(c,"targetBeam"),oldShape=call(head,"getShape");
 try{for(String mode:new String[]{"actual","reverse","duplicate","empty","small-head"}){
 List<Object> candidate=new ArrayList<>(neighbors);if(mode.equals("reverse"))Collections.reverse(candidate);if(mode.equals("duplicate")&&!candidate.isEmpty())candidate.add(candidate.get(0));if(mode.equals("empty"))candidate.clear();
 if(mode.equals("small-head"))set(head,"shape",en("glyph.Shape","NOTEHEAD_BLACK_SMALL"));else set(head,"shape",oldShape);
 set(c,"luArea",call(c,"buildLuArea",(Object)null));Line2D initial=(Line2D)get(c,"theoLine");set(c,"targetBeam",null);
 List<Object> selected=new ArrayList<>(candidate);List<?> selectedGroups=(List<?>)sc("sheet.stem.HeadLinker","lookupBeamGroups",selected,ref,dir,min);
 Point2D target=(Point2D)call(c,"computeTargetPoint",selectedGroups);Object chosen=get(c,"targetBeam");
 rows.add("{\"mode\":\""+mode+"\",\"head\":{\"reference\":"+pt(ref)+",\"x_direction\":"+get(sl,"xDir")+",\"y_direction\":"+dir+",\"slope\":"+call(call(sys,"getSkew"),"getSlope")+",\"slope_margin\":"+get(params,"slopeMargin")+",\"max_head_in_dx\":"+get(params,"maxHeadInDx")+",\"max_head_out_dx\":"+get(params,"maxHeadOutDx")+",\"part_bounds\":"+rect(call(part,"getAreaBounds"))+",\"limit\":null},\"min_distance\":"+min+",\"small_head\":"+call(call(head,"getShape"),"isSmallHead")+",\"candidates\":"+tokens(candidate,bid)+",\"selected_beams\":"+tokens(selected,bid)+",\"selected_groups\":"+tokens(selectedGroups,gid)+",\"target_beam\":"+(chosen==null?"null":bid.get(chosen))+",\"initial_line\":"+ln(initial)+",\"area_path\":"+path(get(c,"luArea"))+",\"area_line\":"+ln((Line2D)get(c,"theoLine"))+",\"target\":"+pt(target)+"}");
 }
 if(!neighbors.isEmpty()){Object b=neighbors.get(0);Line2D near=(Line2D)call(b,"getBorder",en("util.VerticalSide",dir>0?"TOP":"BOTTOM"));Point2D hit=(Point2D)sc("sheet.stem.StemsRetriever","getTargetPt",ref,near,0.);for(int threshold:new int[]{0,min})for(String mode:new String[]{"below","equal","above"}){double y=hit.getY()-dir*threshold;y=mode.equals("below")?Math.nextDown(y):mode.equals("above")?Math.nextUp(y):y;Point2D p=new Point2D.Double(ref.getX(),y);List<Object> one=new ArrayList<>();one.add(b);List<?> sg=(List<?>)sc("sheet.stem.HeadLinker","lookupBeamGroups",one,p,dir,threshold);boundary.add("{\"reference\":"+pt(p)+",\"direction\":"+dir+",\"min_distance\":"+threshold+",\"candidates\":["+bid.get(b)+"],\"selected_beams\":"+tokens(one,bid)+",\"selected_groups\":"+tokens(sg,gid)+"}");}}
 }finally{set(c,"theoLine",oldTheory);set(c,"luArea",oldArea);set(c,"targetBeam",oldTarget);set(head,"shape",oldShape);}
 }}
 Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"native_outputs_used\":false,\"head_builders\":"+count+",\"beams\":["+String.join(",",catalog)+"],\"groups\":["+String.join(",",groupRows)+"],\"cases\":["+String.join(",",rows)+"],\"boundaries\":["+String.join(",",boundary)+"]}");
 }catch(Exception e){throw new RuntimeException(e);}}
}
