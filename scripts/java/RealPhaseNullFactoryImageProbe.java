// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.nio.file.*;import java.util.*;
public class RealPhaseNullFactoryImageProbe extends RealLedgerMembershipImageProbe {
 private final Object actual;
 private static boolean sameObjects(Set<?> before,Set<?> after){return before.size()==after.size()&&before.stream().allMatch(x->after.stream().anyMatch(y->x==y));}
 public RealPhaseNullFactoryImageProbe(Book book,SortedSet<Integer> ids){super(book,ids);actual=book;}
 public void process(){try{
  Object stub=((List<?>)call(actual,"getStubs")).get(0);call(stub,"reachStep",c("step.OmrStep").getField("SYMBOLS").get(null),false);Object sheet=call(stub,"getSheet");Object sys=((List<?>)call(sheet,"getSystems")).get(0);Object sig=call(sys,"getSig"),factory=make("sheet.symbol.InterFactory",sys);Object held=((Collection<?>)call(sig,"inters",c("sig.inter.DynamicsInter"))).iterator().next();Object glyph=call(held,"getGlyph"),staff=call(sys,"getClosestStaff",call(glyph,"getCenter2D"));Set<?> nodes=new LinkedHashSet<>((Set<?>)call(sig,"vertexSet")),edges=new LinkedHashSet<>((Set<?>)call(sig,"edgeSet"));List<String> rows=new ArrayList<>();
  for(String shape:List.of("EIGHTH_set","HW_REST_set","METRO_DOTTED_EIGHTH","METRO_DOTTED_HALF","METRO_DOTTED_QUARTER","METRO_DOTTED_SIXTEENTH","METRO_HALF","METRO_QUARTER","METRO_SIXTEENTH","METRO_WHOLE","TIME_ONE","TIME_ZERO","CLUTTER")){
   Object eval=make("classifier.Evaluation",c("glyph.Shape").getField(shape).get(null),1.0);Object result=call(factory,"create",eval,glyph,staff);boolean unchanged=sameObjects(nodes,(Set<?>)call(sig,"vertexSet"))&&sameObjects(edges,(Set<?>)call(sig,"edgeSet"));
   if(result!=null||!unchanged)throw new Exception("phase null factory changed SIG: "+shape);rows.add("{\"shape\":\""+shape+"\",\"returns_null\":true,\"actual_SIG_unchanged\":true,\"vertex_edge_object_identity_retained\":true}");
  }
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"scope\":\"original automatic InterFactory doCreate explicit CLUTTER or default null, fresh real SYMBOLS owners, declared evaluations on a real glyph; no changes to source settings\",\"cases\":["+String.join(",",rows)+"]}\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
