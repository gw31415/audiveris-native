// SPDX-License-Identifier: AGPL-3.0-or-later
// Development-only GC/identity control, not a product component.
import org.audiveris.omr.sheet.Book;
import java.util.*;
import java.nio.file.*;

import java.lang.ref.*;
public class IdentityControlProbe extends BeamsProbe {
 public IdentityControlProbe(Book b, SortedSet<Integer> ids){super(b,ids);}
 static long gcCount(){long n=System.nanoTime();System.out.println("IDENTITY_MARKER "+n);return n;}
 public void process(){try{
  String mode=System.getProperty("identity.mode","immediate");
  Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");
  call(cls("org.audiveris.omr.sheet.grid.GridStep").getConstructor().newInstance(),"doit",sheet);
  for(Object sys:(List<?>)call(sheet,"getSystems"))call(construct("org.audiveris.omr.sheet.header.HeaderBuilder",new Class<?>[]{cls("org.audiveris.omr.sheet.SystemInfo")},sys),"processHeader");
  call(cls("org.audiveris.omr.sheet.stem.StemSeedsStep").getConstructor().newInstance(),"doit",sheet);
  Object scale=call(sheet,"getScale"),picture=call(sheet,"getPicture"),step=cls("org.audiveris.omr.sheet.beam.BeamsStep").getConstructor().newInstance();
  List<Object> held=null;List<String> stages=new ArrayList<>();long start=gcCount(),built=0,observed=0,dispatched=0;String spotDump="null",indexBefore="null",indexAfter="null";
  if(mode.equals("plain"))call(step,"doit",sheet);
  else{
   Object spotLag=lag("spotLag"),context;
   if(mode.equals("prolog"))context=call(step,"doProlog",sheet);
   else{
    Object sb=construct("org.audiveris.omr.sheet.beam.SpotsBuilder",new Class<?>[]{cls("org.audiveris.omr.sheet.Sheet")},sheet),buffer=call(sb,"getBuffer");
    int beam=(Integer)call(scale,"getItemValue",enumValue("org.audiveris.omr.sheet.Scale$Item","beam"));
    Object small=call(scale,"getItemValue",enumValue("org.audiveris.omr.sheet.Scale$Item","smallBeam"));
    boolean sw=(Boolean)call(call(call(sheet,"getStub"),"getProcessingSwitches"),"getValue",enumValue("org.audiveris.omr.sheet.ProcessingSwitch","smallBeams"));
    if(small!=null)beam=Math.min(beam,(Integer)small);else if(sw)beam=Math.min(beam,(int)Math.rint(beam*(Double)cls("org.audiveris.omr.sheet.beam.BeamsBuilder").getMethod("getCueBeamRatio").invoke(null)));
    List<?> extracted=(List<?>)call(sb,"buildSpots",buffer,null,(double)beam,null);built=gcCount();
    if(mode.startsWith("pin"))held=new ArrayList<>((Collection<?>)call(call(sheet,"getGlyphIndex"),"getEntities"));
    if(mode.contains("inspect")){indexBefore=spots((Collection<?>)call(call(sheet,"getGlyphIndex"),"getEntities"));}
    if(mode.contains("serialize"))spotDump=spots(extracted);
    if(mode.contains("gc"))System.gc();
    observed=gcCount();
    if(mode.contains("inspect")){indexAfter=spots((Collection<?>)call(call(sheet,"getGlyphIndex"),"getEntities"));}
    call(sb,"dispatchSheetSpots",extracted);dispatched=gcCount();
    context=construct("org.audiveris.omr.sheet.beam.BeamsStep$Context",new Class<?>[]{cls("org.audiveris.omr.lag.Lag")},spotLag);
   }
   for(Object sys:(List<?>)call(sheet,"getSystems")){stages.add("{\"system_id\":"+call(sys,"getId")+",\"before_system\":"+spots((Collection<?>)call(sys,"getGroupedGlyphs",enumValue("org.audiveris.omr.glyph.GlyphGroup","BEAM_SPOT")))+"}");call(step,"doSystem",sys,context);}
   call(step,"doEpilog",sheet,context);
  }
  List<String> finals=new ArrayList<>();for(Object sys:(List<?>)call(sheet,"getSystems"))finals.add(finalSystem(sys));
  String finalJson="{\"head_scale\":"+headScale(scale)+",\"head_spots_mask\":"+mask(call(call(picture,"getVerticalTable",enumValue("org.audiveris.omr.sheet.Picture$TableKey","HEAD_SPOTS")),"getBuffer"))+",\"no_staff_after\":"+mask(call(picture,"getSource",enumValue("org.audiveris.omr.sheet.Picture$SourceKey","NO_STAFF")))+",\"systems\":["+String.join(",",finals)+"]}";
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"mode\":"+q(mode)+",\"gc_counts\":["+start+","+built+","+observed+","+dispatched+"],\"index_before\":"+indexBefore+",\"index_after\":"+indexAfter+",\"sheet_spots\":"+spotDump+",\"stages\":["+String.join(",",stages)+"],\"final\":"+finalJson+"}\n");
  Reference.reachabilityFence(held);
 }catch(Exception e){throw new RuntimeException(e);}}
}
