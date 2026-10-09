// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.nio.file.*;import java.util.*;
public class RealPedalCallbacksImageProbe extends RealLedgerMembershipImageProbe {
 private final Object actual;
 public RealPedalCallbacksImageProbe(Book book,SortedSet<Integer> ids){super(book,ids);actual=book;}
 private static String state(Object pedal,Object chord,Object sig,Object edge)throws Exception{
  return "{\"abnormal\":"+call(pedal,"isAbnormal")+",\"manual\":"+call(pedal,"isManual")+",\"relation_manual\":"+call(edge,"isManual")+",\"relation_present\":"+call(sig,"containsEdge",edge)+",\"same_Chord_owner\":"+(call(pedal,"getChord")==chord)+",\"Chord_voice\":"+call(call(chord,"getVoice"),"getId")+",\"Chord_time\":\""+call(chord,"getTimeOffset")+"\",\"Chord_duration\":\""+call(chord,"getDuration")+"\"}";
 }
 public void process(){try{
  Object stub=((List<?>)call(actual,"getStubs")).get(0);call(stub,"reachStep",c("step.OmrStep").getField("RHYTHMS").get(null),false);Object sheet=call(stub,"getSheet");List<String> stages=new ArrayList<>();
  for(Object sys:(List<?>)call(sheet,"getSystems")){
   Object sig=call(sys,"getSig");for(Object pedal:(Collection<?>)call(sig,"inters",c("sig.inter.PedalInter"))){
    Object chord=call(pedal,"getChord");Object voice=call(chord,"getVoice"),slot=call(chord,"getSlot");Object edge=call(sig,"getEdge",chord,pedal);
    stages.add(state(pedal,chord,sig,edge));call(sig,"removeEdge",edge);stages.add(state(pedal,chord,sig,edge));call(edge,"setManual",true);call(sig,"addEdge",chord,pedal,edge);stages.add(state(pedal,chord,sig,edge));
    if(call(chord,"getVoice")!=voice||call(chord,"getSlot")!=slot||call(sig,"getEdge",chord,pedal)!=edge)throw new Exception("retained owner or relation identity changed");
   }
  }
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"scope\":\"fresh actual image RHYTHMS Pedal removed and same manual relation readded; real Chord Voice Slot and relation identity retained\",\"stages\":["+String.join(",",stages)+"],\"actual_identity_retained\":true}\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
