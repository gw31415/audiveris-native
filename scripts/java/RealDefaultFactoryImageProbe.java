// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.nio.file.*;import java.util.*;
public class RealDefaultFactoryImageProbe extends RealLedgerMembershipImageProbe {
 private final Object actual;
 public RealDefaultFactoryImageProbe(Book book,SortedSet<Integer> ids){super(book,ids);actual=book;}
 public void process(){try{
  Object stub=((List<?>)call(actual,"getStubs")).get(0);call(stub,"reachStep",c("step.OmrStep").getField("SYMBOLS").get(null),false);Object sheet=call(stub,"getSheet");Object sys=((List<?>)call(sheet,"getSystems")).get(0);Object sig=call(sys,"getSig"),factory=make("sheet.symbol.InterFactory",sys);Object held=((Collection<?>)call(sig,"inters",c("sig.inter.DynamicsInter"))).iterator().next();Object glyph=call(held,"getGlyph"),staff=call(sys,"getClosestStaff",call(glyph,"getCenter2D"));Set<?> nodes=new LinkedHashSet<>((Set<?>)call(sig,"vertexSet")),edges=new LinkedHashSet<>((Set<?>)call(sig,"edgeSet"));List<String> rows=new ArrayList<>();
  for(String shape:List.of("PLUCK_P","PLUCK_I","PLUCK_M","PLUCK_A","DIGIT_0","DIGIT_1","DIGIT_2","DIGIT_3","DIGIT_4","DIGIT_5","ROMAN_I","ROMAN_II","ROMAN_III","ROMAN_IV","ROMAN_V","ROMAN_VI","ROMAN_VII","ROMAN_VIII","ROMAN_IX","ROMAN_X","ROMAN_XI","ROMAN_XII","TREMOLO_1","TREMOLO_2","TREMOLO_3","PLAYING_OPEN","PLAYING_HALF_OPEN","PLAYING_CLOSED")){
   Object eval=make("classifier.Evaluation",c("glyph.Shape").getField(shape).get(null),1.0);Object result=call(factory,"create",eval,glyph,staff);boolean unchanged=nodes.equals(call(sig,"vertexSet"))&&edges.equals(call(sig,"edgeSet"));
   if(result!=null||!unchanged)throw new Exception("default null factory changed SIG: "+shape);rows.add("{\"shape\":\""+shape+"\",\"returns_null\":true,\"actual_SIG_unchanged\":true}");
  }
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"scope\":\"original default switches, fresh real SYMBOLS owners, declared evaluations on a real glyph; no changes to source settings\",\"cases\":["+String.join(",",rows)+"]}\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
