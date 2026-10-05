// SPDX-License-Identifier: AGPL-3.0-or-later
// Direct original getCandidateFilaments on a real GRID system; declared sections.
import org.audiveris.omr.sheet.Book;
import java.util.*;import java.lang.reflect.*;import java.nio.file.*;import java.awt.geom.*;
public class LedgerCandidateControlProbe extends StickFactoryControlProbe {
 public LedgerCandidateControlProbe(Book b,SortedSet<Integer> ids){super(b,ids);}
 static Object make(String n,Object...a)throws Exception{for(Constructor<?> c:Class.forName("org.audiveris.omr."+n).getDeclaredConstructors())if(c.getParameterCount()==a.length){c.setAccessible(true);try{return c.newInstance(a);}catch(IllegalArgumentException e){}}throw new NoSuchMethodException(n);}
 static void setField(Object o,String n,Object v)throws Exception{Field f=o.getClass().getDeclaredField(n);f.setAccessible(true);f.set(o,v);}
 Object ledgerSection(int id,int y,int x,int width,int height)throws Exception{int[][] r=new int[height][2];for(int i=0;i<height;i++)r[i]=new int[]{x,width};return section(id,false,y,r);}
 String beam(Object b)throws Exception{PathIterator it=((Area)call(b,"getArea")).getPathIterator(null);List<String> seg=new ArrayList<>();double[] first=null,previous=null;while(!it.isDone()){double[] xy=new double[6];int t=it.currentSegment(xy);double[] pt={xy[0],xy[1]};if(t==0){first=pt;previous=pt;}else if(t==1){seg.add("["+Arrays.toString(previous)+","+Arrays.toString(pt)+"]");previous=pt;}else if(t==4){seg.add("["+Arrays.toString(previous)+","+Arrays.toString(first)+"]");}else throw new AssertionError("curved beam");it.next();}return "{\"bounds\":"+rect(call(b,"getBounds"))+",\"segments\":["+String.join(",",seg)+"]}";}
 String observeLedger(String name,List<Object> sections,int il,int fore,List<Object> beams)throws Exception{
  call(sheet,"setScale",make("sheet.Scale",make("sheet.Scale$InterlineScale",il,il,il),make("sheet.Scale$LineScale",fore,fore,fore),null,null,null));
  Object builder=make("sheet.ledger.LedgersBuilder",system);setField(builder,"sortedSystemBeams",beams);
  Method m=builder.getClass().getDeclaredMethod("getCandidateFilaments",List.class);m.setAccessible(true);List<?> sticks=(List<?>)m.invoke(builder,new ArrayList<>(sections));List<String> out=new ArrayList<>(),bs=new ArrayList<>();
  for(Object b:beams){if(!(Boolean)call(b,"isGood"))throw new AssertionError("beam not good");bs.add(beam(b));}
  Object old=vertical;vertical=horizontal;try{for(Object f:sticks)out.add(filament(f));}finally{vertical=old;}
  return "{\"name\":"+q(name)+",\"interline\":"+il+",\"fore\":"+fore+",\"sections\":"+StemSeedsProbe.sectionList(sections)+",\"good_beams\":["+String.join(",",bs)+"],\"candidates\":["+String.join(",",out)+"]}";
 }
 public void process(){try{
  sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");call(make("sheet.grid.GridStep"),"doit",sheet);system=((List<?>)call(sheet,"getSystems")).get(0);Class<?> o=Class.forName("org.audiveris.omr.run.Orientation");vertical=o.getField("VERTICAL").get(null);horizontal=o.getField("HORIZONTAL").get(null);List<String> cases=new ArrayList<>();
  for(int il:new int[]{11,16,17,24})for(int fore:new int[]{1,2,3}){
   int max=Math.min((int)Math.rint(3.25*fore),(int)Math.rint(.4*il));
   for(int length:new int[]{il-1,il,il+1,20*il,20*il+1})cases.add(observeLedger("length-"+il+"-"+fore+"-"+length,List.of(ledgerSection(1,200,200,length,1)),il,fore,List.of()));
   for(int thickness:new int[]{max,max+1})cases.add(observeLedger("thickness-"+il+"-"+fore+"-"+thickness,List.of(ledgerSection(1,200,200,50,thickness)),il,fore,List.of()));
  }
  for(int overlap:new int[]{44,45,46})cases.add(observeLedger("side-ratio-"+overlap,List.of(ledgerSection(1,200,200,50,1),ledgerSection(2,201,200,overlap,2)),16,2,List.of()));
  for(double y:new double[]{195,198.5,200,201.5,205}){Object b=make("sig.inter.BeamInter",make("sig.inter.AbstractBeamInter$Impacts",1.,1.,1.,1.,1.,1.),new Line2D.Double(190,y,270,y+2),4.);cases.add(observeLedger("beam-midpoint-"+y,List.of(ledgerSection(1,200,200,50,1)),16,2,List.of(b)));}
  cases.add(observeLedger("empty",List.of(),16,2,List.of()));
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"native_outputs_used\":false,\"scope\":\"Unmodified LedgersBuilder constructor/getCandidateFilaments and original StickFactory/purges; real GRID system, supplied sections and declared good full beams; no LedgerInter acceptance or real-score rest claim.\",\"cases\":["+String.join(",",cases)+"]}\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
