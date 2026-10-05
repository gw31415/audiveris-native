// SPDX-License-Identifier: AGPL-3.0-or-later
// Direct original lookupOtherHeads in the saved HEADS inspectStems context.
import org.audiveris.omr.sheet.Book;import java.util.*;import java.nio.file.*;import java.awt.geom.*;
public class StemImageLinkersProbe extends StemImageSourcesProbe {
 private final Object book;public StemImageLinkersProbe(Book b,SortedSet<Integer> ids){super(b,ids);book=b;}
 static String point(Object p){Point2D x=(Point2D)p;return "["+x.getX()+","+x.getY()+"]";}
 static Object headOf(Object c)throws Exception{return get(get(c,"this$1"),"this$0");}
 static Object owner(Object c)throws Exception{return get(headOf(c),"head");}
 static String corner(Object c)throws Exception{Object h=owner(c),g=call(c,"getStump");return "{\"head_bounds\":"+rect(call(h,"getBounds"))+",\"shape\":\""+call(h,"getShape")+"\",\"side\":\""+call(c,"getHorizontalSide")+"\",\"vertical\":\""+get(c,"vSide")+"\",\"reference\":"+point(call(c,"getReferencePoint"))+",\"stump\":"+(g==null?"null":physical(g))+"}";}
 static Object cornerOf(Object h,String side,String v)throws Exception{return call(call(h,"getLinker"),"getCornerLinker",en("util.HorizontalSide",side),en("util.VerticalSide",v));}
 static String heads(List<?> hs,Object sig)throws Exception{List<String>r=new ArrayList<>();for(Object h:hs){List<String>c=new ArrayList<>();for(String s:new String[]{"LEFT","RIGHT"})for(String v:new String[]{"TOP","BOTTOM"})c.add(corner(cornerOf(h,s,v)));r.add("{\"id\":"+call(h,"getId")+",\"head_bounds\":"+rect(call(h,"getBounds"))+",\"center\":"+point(call(h,"getCenter"))+",\"shape\":\""+call(h,"getShape")+"\",\"removed\":"+call(h,"isRemoved")+",\"competing_ids\":"+ids((Collection<?>)call(sig,"getCompetingInters",h))+",\"corners\":["+String.join(",",c)+"]}");}return "["+String.join(",",r)+"]";}
 static String row(Object c,Object p,String name)throws Exception{List<String>r=new ArrayList<>();for(Object x:(List<?>)call(c,"lookupOtherHeads"))r.add(corner(x));return "{\"name\":\""+name+"\",\"source_id\":"+call(owner(c),"getId")+",\"head\":"+corner(c)+",\"min_head_head_dy\":"+get(p,"minHeadHeadDy")+",\"area_path\":"+path(get(c,"luArea"))+",\"selected\":["+String.join(",",r)+"]}";}
 public void process(){try{
 Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");List<String>ss=new ArrayList<>(),controls=new ArrayList<>();Object firstR=null,firstP=null,firstSig=null;List<?> firstHeads=null;int count=0,selected=0;
 for(Object sys:(List<?>)call(sheet,"getSystems")){Object r=inspect(sys,sheet),p=get(r,"params"),sig=call(sys,"getSig");List<?> hs=(List<?>)get(r,"systemHeads");if(firstR==null){firstR=r;firstP=p;firstSig=sig;firstHeads=hs;}List<String>cases=new ArrayList<>();for(Object h:hs)for(String side:new String[]{"LEFT","RIGHT"})for(String v:new String[]{"TOP","BOTTOM"}){Object c=cornerOf(h,side,v);cases.add(row(c,p,"actual"));count++;selected+=((List<?>)call(c,"lookupOtherHeads")).size();}ss.add("{\"system\":"+call(sys,"getId")+",\"head_catalog\":"+heads(hs,sig)+",\"cases\":["+String.join(",",cases)+"]}");}
 String catalog=heads(firstHeads,firstSig);Object source=null,target=null;for(Object h:firstHeads)if(call(h,"getShape").toString().equals("NOTEHEAD_BLACK")){if(source==null)source=h;else if(target==null&&!((Collection<?>)call(firstSig,"getCompetingInters",source)).contains(h))target=h;}
 Object c=cornerOf(source,"LEFT","TOP"),tc=cornerOf(target,"LEFT","TOP");Point2D ref=(Point2D)call(tc,"getReferencePoint"),center=(Point2D)call(target,"getCenter");double x=ref.getX(),y=ref.getY(),cy=center.getY();
 set(c,"luArea",new Area());controls.add(row(c,firstP,"empty-area"));
 for(int min:new int[]{0,3})for(double dy:new double[]{Math.nextDown(cy+min),cy+min,Math.nextUp(cy+min)}){set(firstP,"minHeadHeadDy",min);set(c,"refPt",new Point2D.Double(x,dy));set(c,"luArea",new Area(new Rectangle2D.Double(x-20,y-20,40,40)));controls.add(row(c,firstP,"signed-min-dy-"+min+"-"+dy));}
 set(firstP,"minHeadHeadDy",3);set(c,"refPt",new Point2D.Double(x,cy+10));
 for(String edge:new String[]{"left","right","top","bottom"}){double ax=x-1,ay=y-1;if(edge.equals("left"))ax=x;if(edge.equals("right"))ax=x-2;if(edge.equals("top"))ay=y;if(edge.equals("bottom"))ay=y-2;set(c,"luArea",new Area(new Rectangle2D.Double(ax,ay,2,2)));controls.add(row(c,firstP,"reference-edge-"+edge));}
 set(c,"luArea",new Area(new Rectangle2D.Double(x-20,y-20,40,40)));set(source,"shape",en("glyph.Shape","NOTEHEAD_VOID"));controls.add(row(c,firstP,"different-duration"));set(source,"shape",en("glyph.Shape","NOTEHEAD_BLACK"));
 set(firstP,"minHeadHeadDy",0);set(c,"luArea",new Area(new Rectangle2D.Double(0,0,2000,3000)));set(c,"refPt",new Point2D.Double(0,3000));controls.add(row(c,firstP,"self-competing-and-parallel-heads"));
 set(firstR,"systemHeads",new ArrayList<>());controls.add(row(c,firstP,"empty-heads"));
 Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"native_outputs_used\":false,\"original_phase\":\"inspectStems_before_linkStems_and_finalizeStems\",\"corners\":"+count+",\"selected_other_head_linkers\":"+selected+",\"systems\":["+String.join(",",ss)+"],\"control_head_catalog\":"+catalog+",\"controls\":["+String.join(",",controls)+"]}");
 }catch(Exception e){throw new RuntimeException(e);}}
}
