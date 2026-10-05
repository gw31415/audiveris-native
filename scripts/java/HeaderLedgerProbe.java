// SPDX-License-Identifier: AGPL-3.0-or-later
// Original same-image STEMS, saved XML callback then actual reduceFoundations.
import org.audiveris.omr.sheet.Book;import java.util.*;import java.nio.file.*;import java.lang.reflect.*;
public class HeaderLedgerProbe extends StemImageScheduleProbe {
 final Object book;public HeaderLedgerProbe(Book b,SortedSet<Integer> ids){super(b,ids);book=b;}
 static int indexOf(Object staff,Object ledger)throws Exception {
  for(Object entry:((Map<?,?>)call(staff,"getLedgerMap")).entrySet()) {
   Map.Entry<?,?> e=(Map.Entry<?,?>)entry;
   if(((Collection<?>)e.getValue()).contains(ledger))return ((Number)e.getKey()).intValue();
  }
  throw new IllegalStateException("ledger absent from original restored staff map");
 }
 public static void main(String[] a)throws Exception {
  Object b=Class.forName("org.audiveris.omr.sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(a[0]));
  new HeaderLedgerProbe((Book)b,new TreeSet<Integer>()).process();
 }
 public void process(){try {
  Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig"),staff=((List<?>)call(sys,"getStaves")).get(0);
  Object reducer=Class.forName("org.audiveris.omr.sig.SigReducer").getConstructor(Class.forName("org.audiveris.omr.sheet.SystemInfo"),boolean.class).newInstance(sys,true);
  Method supported=reducer.getClass().getDeclaredMethod("ledgerHasHeadOrLedger",Class.forName("org.audiveris.omr.sheet.Staff"),int.class,Class.forName("org.audiveris.omr.sig.inter.LedgerInter"),List.class);supported.setAccessible(true);
  List<Object> ledgers=new ArrayList<>(),heads=new ArrayList<>();for(Object n:(Collection<?>)call(sig,"vertexSet")){
   if(n.getClass().getSimpleName().equals("LedgerInter"))ledgers.add(n);
   if(n.getClass().getSimpleName().equals("HeadInter"))heads.add(n);
  }
  heads.sort(Comparator.comparingInt(n->{try{return ((java.awt.Rectangle)call(n,"getBounds")).x;}catch(Exception e){throw new RuntimeException(e);}}));
  List<String> before=new ArrayList<>();for(Object n:ledgers)before.add("{\"bounds\":"+rect(call(n,"getBounds"))+",\"index\":"+indexOf(staff,n)+",\"grade\":"+call(n,"getGrade")+",\"glyph\":"+physical(call(n,"getGlyph"))+",\"has_head_or_next_ledger\":"+supported.invoke(reducer,staff,indexOf(staff,n),n,heads)+"}");
  call(reducer,"reduceFoundations");List<String> after=new ArrayList<>();for(Object n:ledgers)after.add("{\"bounds\":"+rect(call(n,"getBounds"))+",\"removed\":"+call(n,"isRemoved")+"}");
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"scope\":\"original same-image saved STEMS XML reload and actual reduceFoundations; orphan predicate uses restored staff ledger map index (LedgerInter index is transient)\",\"native_outputs_used\":false,\"before_reduction\":["+String.join(",",before)+"],\"after_reduction\":["+String.join(",",after)+"]}\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
