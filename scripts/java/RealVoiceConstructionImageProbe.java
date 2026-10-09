// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.nio.file.*;import java.util.*;
public class RealVoiceConstructionImageProbe extends RealLedgerMembershipImageProbe {
 private final Object actual;
 public RealVoiceConstructionImageProbe(Book book,SortedSet<Integer> ids){super(book,ids);actual=book;}
 private static String nullable(Object v){return v==null?"null":"\""+v+"\"";}
 private static String state(Object sys,Object part,Object measure,Object seed,Object voice,String phase)throws Exception {
 List<String> members=new ArrayList<>();for(Object ch:(List<?>)call(voice,"getChords"))members.add(rect(call(ch,"getBounds")));
 List<String> chords=new ArrayList<>();for(Object ch:(Collection<?>)call(measure,"getHeadChords"))chords.add("{\"bounds\":"+rect(call(ch,"getBounds"))+",\"assigned_to_this_actual_Voice\":"+(call(ch,"getVoice")==voice)+",\"time\":"+nullable(call(ch,"getTimeOffset"))+",\"slot\":"+(call(ch,"getSlot")==null?"null":call(call(ch,"getSlot"),"getId"))+"}");
 Object staff=call(voice,"getStartingStaff");return "{\"phase\":\""+phase+"\",\"system\":"+call(sys,"getId")+",\"part\":"+call(part,"getId")+",\"measure_index\":"+(((List<?>)call(part,"getMeasures")).indexOf(measure)+1)+",\"seed_bounds\":"+rect(call(seed,"getBounds"))+",\"kind\":\""+call(voice,"getKind")+"\",\"id\":"+call(voice,"getId")+",\"starting_staff\":"+call(staff,"getId")+",\"same_actual_Part_Staff\":"+(call(staff,"getPart")==part)+",\"same_actual_Measure\":"+(call(voice,"getMeasure")==measure)+",\"registered\":"+((List<?>)call(measure,"getVoices")).contains(voice)+",\"termination\":"+nullable(call(voice,"getTermination"))+",\"chords\":["+String.join(",",members)+"],\"all_measure_chords\":["+String.join(",",chords)+"]}";
 }
 public void process(){try{Object stub=((List<?>)call(actual,"getStubs")).get(0);call(stub,"reachStep",c("step.OmrStep").getField("LINKS").get(null),false);Object sheet=call(stub,"getSheet");List<String> rows=new ArrayList<>();Object controlSys=null,controlPart=null,controlMeasure=null,controlSeed=null;
 for(Object sys:(List<?>)call(sheet,"getSystems"))for(Object part:(List<?>)call(sys,"getParts"))for(Object measure:(List<?>)call(part,"getMeasures")){
 List<Object> chords=new ArrayList<>((Collection<?>)call(measure,"getHeadChords"));chords.sort(Comparator.comparingInt(ch->{try{return ((java.awt.Rectangle)call(ch,"getBounds")).x;}catch(Exception e){throw new RuntimeException(e);}}));
 for(Object staff:(List<?>)call(part,"getStaves")){Object seed=null;for(Object ch:chords)if(call(ch,"getTopStaff")==staff){seed=ch;break;}if(seed==null)continue;
 Object voice=c("sheet.rhythm.Voice").getConstructor(c("sig.inter.AbstractChordInter"),c("sheet.rhythm.Measure")).newInstance(seed,measure);rows.add(state(sys,part,measure,seed,voice,"CONSTRUCTED"));call(measure,"addVoice",voice);rows.add(state(sys,part,measure,seed,voice,"REGISTERED"));if(controlSeed==null && call(voice,"getKind").toString().equals("LOW")){controlSys=sys;controlPart=part;controlMeasure=measure;controlSeed=seed;}
 }
 }
 Object retired=null;for(int i=0;i<5;i++){Object voice=c("sheet.rhythm.Voice").getConstructor(c("sig.inter.AbstractChordInter"),c("sheet.rhythm.Measure")).newInstance(controlSeed,controlMeasure);rows.add(state(controlSys,controlPart,controlMeasure,controlSeed,voice,"LOW_RECONSTRUCTED"));call(controlMeasure,"addVoice",voice);rows.add(state(controlSys,controlPart,controlMeasure,controlSeed,voice,"LOW_REGISTERED"));if(i==0)retired=voice;}
 call(retired,"resetChords");rows.add(state(controlSys,controlPart,controlMeasure,controlSeed,retired,"OLD_LOW_EMPTIED"));call(controlMeasure,"purgeVoices");rows.add(state(controlSys,controlPart,controlMeasure,controlSeed,retired,"OLD_LOW_RETIRED"));Object replacement=c("sheet.rhythm.Voice").getConstructor(c("sig.inter.AbstractChordInter"),c("sheet.rhythm.Measure")).newInstance(controlSeed,controlMeasure);rows.add(state(controlSys,controlPart,controlMeasure,controlSeed,replacement,"LOW_NUMBER_REUSED"));call(controlMeasure,"addVoice",replacement);rows.add(state(controlSys,controlPart,controlMeasure,controlSeed,replacement,"LOW_REPLACEMENT_REGISTERED"));
 Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"scope\":\"fresh unchanged original LINKS; declared first abscissa Chord per actual Part Staff Voice constructors, registration and same-Measure BeamGroup propagation before rhythms, no native input\",\"states\":["+String.join(",",rows)+"]}\n");}catch(Exception e){throw new RuntimeException(e);}}
}
