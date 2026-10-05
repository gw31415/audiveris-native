// SPDX-License-Identifier: AGPL-3.0-or-later
// Original lookupChunks registration/removeAll prefix on actual raw chunks.
import org.audiveris.omr.sheet.Book;
import java.util.*;import java.nio.file.*;
public class StemChunkRegistrationProbe extends StemPartsLookupProbe {
 private final Object originalBook;
 public StemChunkRegistrationProbe(Book b,SortedSet<Integer> ids){super(b,ids);originalBook=b;}
 static List<Object> raw(Object sb,Object sys)throws Exception {
  Object f=Class.forName("org.audiveris.omr.glyph.dynamic.StickFactory").getConstructor(Class.forName("org.audiveris.omr.run.Orientation"),Class.forName("org.audiveris.omr.sheet.SystemInfo"),Class.forName("org.audiveris.omr.glyph.dynamic.FilamentIndex"),java.util.function.Predicate.class,int.class,int.class,double.class).newInstance(en("run.Orientation","VERTICAL"),sys,call(call(sys,"getSheet"),"getFilamentIndex"),null,get(get(sb,"params"),"maxStemThickness"),0,0.4);
  List<Object> r=new ArrayList<>();for(Object stick:(List<?>)call(f,"retrieveSticks",call(sb,"lookupVerticalSections"),call(sb,"lookupHorizontalSections")))r.add(call(stick,"toGlyph",(Object)null));return r;
 }
 static String tokens(Collection<?> items,IdentityHashMap<Object,Integer> tags){List<String>r=new ArrayList<>();for(Object g:items)r.add(""+tags.get(g));return "["+String.join(",",r)+"]";}
 static String observe(Object index,List<Object> raw,List<Object> existing,IdentityHashMap<Object,Integer> tags,List<Object> held,String mode)throws Exception {
  int before=tags.size();List<String>phys=new ArrayList<>();List<Object> registered=new ArrayList<>();
  for(Object g:raw){phys.add(physical(g));Object c=call(index,"registerOriginal",g);if(!tags.containsKey(c)){tags.put(c,tags.size()+1);held.add(c);}registered.add(c);}
  String all=tokens(registered,tags),ex=tokens(existing,tags);List<Object>kept=new ArrayList<>(registered);kept.removeAll(existing);
  if(((Collection<?>)call(index,"getEntities")).size()!=tags.size())throw new IllegalStateException("unheld original glyph");
  return "{\"mode\":\""+mode+"\",\"raw_glyphs\":["+String.join(",",phys)+"],\"existing_ids\":"+ex+",\"registered_ids\":"+all+",\"retained_ids\":"+tokens(kept,tags)+",\"new_count\":"+(tags.size()-before)+",\"registry_count\":"+tags.size()+"}";
 }
 public void process(){try{
  Object sheet=call(((List<?>)call(originalBook,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0);
  Object retr=Class.forName("org.audiveris.omr.sheet.stem.StemsRetriever").getConstructor(Class.forName("org.audiveris.omr.sheet.SystemInfo")).newInstance(sys);call(retr,"process");
  Set<Object>seen=Collections.newSetFromMap(new IdentityHashMap<>());List<Object>builders=new ArrayList<>();
  for(Object head:(List<?>)get(retr,"systemHeads")){Object linker=call(head,"getLinker");if(linker==null)continue;for(String side:new String[]{"LEFT","RIGHT"})for(String vert:new String[]{"TOP","BOTTOM"}){Object c=call(linker,"getCornerLinker",en("util.HorizontalSide",side),en("util.VerticalSide",vert)),sb=get(c,"sb");if(sb!=null&&seen.add(sb))builders.add(sb);}}
  for(Object beam:(List<?>)get(retr,"systemBeams")){Object linker=call(beam,"getLinker");if(linker==null)continue;for(Object side:(List<?>)get(linker,"allBLinkers"))for(Object c:(Collection<?>)call(side,"getHalfLinkers")){Object sb=get(c,"sb");if(sb!=null&&seen.add(sb))builders.add(sb);}}
  List<String>groups=new ArrayList<>();int total=0;
  for(Object sb:builders){List<Object> raw=raw(sb,sys);total+=raw.size();Object index=Class.forName("org.audiveris.omr.glyph.GlyphIndex").getConstructor().newInstance();call(index,"initTransients",sheet);IdentityHashMap<Object,Integer>tags=new IdentityHashMap<>();List<Object>held=new ArrayList<>();List<String>rows=new ArrayList<>();
   rows.add(observe(index,raw,new ArrayList<>(),tags,held,"fresh"));List<Object>all=new ArrayList<>(held),first=new ArrayList<>();if(!held.isEmpty()){first.add(held.get(0));first.add(held.get(0));}
   rows.add(observe(index,raw,first,tags,held,"exclude-first-alias-twice"));rows.add(observe(index,raw,all,tags,held,"exclude-all"));List<Object>dup=new ArrayList<>(raw);if(!raw.isEmpty())dup.add(raw.get(0));rows.add(observe(index,dup,new ArrayList<>(),tags,held,"duplicate-raw-alias"));
   groups.add("{\"start_kind\":\""+get(sb,"startLinker").getClass().getSimpleName()+"\",\"cases\":["+String.join(",",rows)+"]}");
  }
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"native_outputs_used\":false,\"JVM_ids_normalized_to_canonical_identity_tokens\":true,\"scope\":\"LookupChunks registerOriginal/removeAll prefix only, fresh legitimate sheet-linked index per actual builder; no whole filter/cache/SIG or page registration claim\",\"builders\":"+builders.size()+",\"actual_raw_glyphs\":"+total+",\"groups\":["+String.join(",",groups)+"]}");
 }catch(Exception e){throw new RuntimeException(e);}}
}
