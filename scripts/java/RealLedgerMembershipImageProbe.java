// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.step.RunClass;
import org.audiveris.omr.sheet.Book;
import java.nio.file.*;
import java.util.*;
import java.awt.Rectangle;
import java.lang.reflect.*;
public class RealLedgerMembershipImageProbe extends RunClass {
 private final Object actualBook;
 public RealLedgerMembershipImageProbe(Book book,SortedSet<Integer> ids){super(book,ids);actualBook=book;}
 static Class<?> c(String name)throws Exception{return Class.forName("org.audiveris.omr."+name);}
 static Object call(Object o,String name,Object... args)throws Exception{for(Class<?> k=o.getClass();k!=null;k=k.getSuperclass())for(Method m:k.getDeclaredMethods())if(m.getName().equals(name)&&m.getParameterCount()==args.length){m.setAccessible(true);try{return m.invoke(o,args);}catch(IllegalArgumentException e){}}throw new NoSuchMethodException(name);}
 static Object make(String name,Object...args)throws Exception{for(Constructor<?> ctor:c(name).getConstructors())try{return ctor.newInstance(args);}catch(IllegalArgumentException e){}throw new NoSuchMethodException(name);}
 static String rect(Object value){if(value==null)return "null";Rectangle r=(Rectangle)value;return "["+r.x+","+r.y+","+r.width+","+r.height+"]";}
 static String glyph(Object g)throws Exception{if(g==null)return "null";Object table=call(g,"getRunTable");List<String> runs=new ArrayList<>();for(int p=0;p<((Number)call(table,"getSize")).intValue();p++){Iterator<?> it=(Iterator<?>)call(table,"iterator",p);while(it.hasNext()){Object r=it.next();runs.add("["+p+","+call(r,"getStart")+","+call(r,"getLength")+"]");}}return "{\"bounds\":"+rect(call(g,"getBounds"))+",\"orientation\":\""+call(table,"getOrientation")+"\",\"runs\":["+String.join(",",runs)+"]}";}
 static String state(Object sheet)throws Exception{
  List<String> systems=new ArrayList<>();for(Object sys:(List<?>)call(sheet,"getSystems")){
   List<String> maps=new ArrayList<>(),bodies=new ArrayList<>();Map<Object,Integer> names=new IdentityHashMap<>();
   for(Object staff:(List<?>)call(sys,"getStaves"))for(Object row:((Map<?,?>)call(staff,"getLedgerMap")).values())for(Object ledger:(List<?>)row)if(!names.containsKey(ledger)){int id=names.size()+1;names.put(ledger,id);Object owner=call(ledger,"getStaff");bodies.add("{\"label\":"+id+",\"bounds\":"+rect(call(ledger,"getBounds"))+",\"index\":"+call(ledger,"getIndex")+",\"owner_staff\":"+(owner==null?"null":call(owner,"getId"))+",\"removed\":"+call(ledger,"isRemoved")+",\"glyph\":"+glyph(call(ledger,"getGlyph"))+"}");}
   for(Object staff:(List<?>)call(sys,"getStaves")){List<String> lines=new ArrayList<>();for(var entry:((Map<?,?>)call(staff,"getLedgerMap")).entrySet()){List<String> ids=new ArrayList<>();for(Object n:(List<?>)entry.getValue())ids.add(names.get(n).toString());lines.add("{\"index\":"+entry.getKey()+",\"members\":["+String.join(",",ids)+"]}");}maps.add("{\"staff\":"+call(staff,"getId")+",\"lines\":["+String.join(",",lines)+"]}");}
   systems.add("{\"system\":"+call(sys,"getId")+",\"bodies\":["+String.join(",",bodies)+"],\"maps\":["+String.join(",",maps)+"]}");
  }return "["+String.join(",",systems)+"]";
 }
 public void process(){try{
  Object sheet=call(((List<?>)call(actualBook,"getStubs")).get(0),"getSheet");for(String step:List.of("sheet.grid.GridStep","sheet.header.HeadersStep","sheet.stem.StemSeedsStep","sheet.beam.BeamsStep","sheet.ledger.LedgersStep"))call(make(step),"doit",sheet);String ledgers=state(sheet);call(make("sheet.note.HeadsStep"),"doit",sheet);String heads=state(sheet);
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"scope\":\"fresh unchanged original image SCALE->GRID->HEADERS->STEM_SEEDS->BEAMS->LEDGERS/post-analysis->HEADS before STEMS/foundation shared-ledger repair; actual object memberships, no native outputs\",\"ledger_stage\":"+ledgers+",\"head_stage\":"+heads+"}\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
