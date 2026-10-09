// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.nio.file.*;import java.util.*;import java.lang.reflect.*;
public class RealPageTimeRangesImageProbe extends RealLedgerMembershipImageProbe {
 private final Object actual;
 public RealPageTimeRangesImageProbe(Book book,SortedSet<Integer> ids){super(book,ids);actual=book;}
 private static Object field(Object o,String name)throws Exception{Field f=o.getClass().getDeclaredField(name);f.setAccessible(true);return f.get(o);}
 public void process(){try{Object stub=((List<?>)call(actual,"getStubs")).get(0);call(stub,"reachStep",c("step.OmrStep").getField("LINKS").get(null),false);Object sheet=call(stub,"getSheet"),page=((List<?>)call(sheet,"getPages")).get(0),rhythm=make("sheet.rhythm.PageRhythm",page);if(!(Boolean)call(rhythm,"populateTimeSignatures"))throw new IllegalStateException("time ranges failed");List<String> rows=new ArrayList<>();for(Object range:(List<?>)field(rhythm,"ranges")){Object ts=field(range,"ts"),duration=field(range,"duration");String signature=ts==null?"null":"{\"bounds\":"+rect(call(ts,"getBounds"))+",\"system\":"+call(call(ts,"getSig"),"getSystem").getClass().getMethod("getId").invoke(call(call(ts,"getSig"),"getSystem"))+",\"staff\":"+call(call(ts,"getStaff"),"getId")+"}";rows.add("{\"start\":"+field(range,"startSN")+",\"stop\":"+field(range,"stopSN")+",\"duration\":"+(duration==null?"null":"\""+duration+"\"")+",\"signature\":"+signature+"}");}Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"scope\":\"fresh unchanged image LINKS actual PageRhythm.populateTimeSignatures, no native input\",\"ranges\":["+String.join(",",rows)+"]}\n");}catch(Exception e){throw new RuntimeException(e);}}
}
