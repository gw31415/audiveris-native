// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.nio.file.*;import java.util.*;
public class RealVoiceDistanceImageProbe extends RealLedgerMembershipImageProbe {
 private final Object actual;
 public RealVoiceDistanceImageProbe(Book book,SortedSet<Integer> ids){super(book,ids);actual=book;}
 private static String distances(Object sheet,String stage)throws Exception{
 Object distance=c("sheet.rhythm.VoiceDistance$Separated").getConstructor(c("sheet.Scale")).newInstance(call(sheet,"getScale"));List<String> rows=new ArrayList<>();
 for(Object sys:(List<?>)call(sheet,"getSystems"))for(Object part:(List<?>)call(sys,"getParts"))for(Object measure:(List<?>)call(part,"getMeasures")){
 List<Object> chords=new ArrayList<>((Collection<?>)call(measure,"getHeadChords"));chords.sort(Comparator.comparingInt(ch->{try{return ((java.awt.Rectangle)call(ch,"getBounds")).x;}catch(Exception e){throw new RuntimeException(e);}}));
 for(Object left:chords)for(Object right:chords){StringBuilder details=new StringBuilder();Object ls=call(left,"getTopStaff"),rs=call(right,"getTopStaff");rows.add("{\"system\":"+call(sys,"getId")+",\"part\":"+call(part,"getId")+",\"measure_index\":"+(((List<?>)call(part,"getMeasures")).indexOf(measure)+1)+",\"left_bounds\":"+rect(call(left,"getBounds"))+",\"right_bounds\":"+rect(call(right,"getBounds"))+",\"left_staff\":"+call(ls,"getId")+",\"right_staff\":"+call(rs,"getId")+",\"distance\":"+call(distance,"getDistance",left,right,details)+",\"details\":\""+details+"\"}");}}
 return "{\"stage\":\""+stage+"\",\"pairs\":["+String.join(",",rows)+"]}";
 }
 public void process(){try{Object stub=((List<?>)call(actual,"getStubs")).get(0);call(stub,"reachStep",c("step.OmrStep").getField("LINKS").get(null),false);Object sheet=call(stub,"getSheet");List<String> stages=new ArrayList<>();stages.add(distances(sheet,"UNASSIGNED"));
 for(Object sys:(List<?>)call(sheet,"getSystems"))for(Object part:(List<?>)call(sys,"getParts"))for(Object measure:(List<?>)call(part,"getMeasures")){
 List<Object> chords=new ArrayList<>((Collection<?>)call(measure,"getHeadChords"));chords.sort(Comparator.comparingInt(ch->{try{return ((java.awt.Rectangle)call(ch,"getBounds")).x;}catch(Exception e){throw new RuntimeException(e);}}));
 for(Object staff:(List<?>)call(part,"getStaves")){Object seed=null;for(Object ch:chords)if(call(ch,"getTopStaff")==staff){seed=ch;break;}if(seed==null)continue;Object voice=c("sheet.rhythm.Voice").getConstructor(c("sig.inter.AbstractChordInter"),c("sheet.rhythm.Measure")).newInstance(seed,measure);call(measure,"addVoice",voice);}}
 stages.add(distances(sheet,"FIRST_ACTUAL_STAFF_VOICES"));Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"scope\":\"fresh unchanged LINKS all actual same-Measure HeadChord pairs; separated distance before and after declared first-Staff Voice constructors, no native input\",\"stages\":["+String.join(",",stages)+"]}\n");}catch(Exception e){throw new RuntimeException(e);}}
}
