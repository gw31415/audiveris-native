// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

public class GroupedSlotClockProbe extends MappingRetryProbe {
    static Object inspectClock(Object measure,String name) throws Exception {
        call(measure,"resetRhythm");
        List<Object> chords=new ArrayList<>((Collection<?>)call(measure,"getStandardChords"));
        for(int i=0;i<2;i++)call(measure,"addVoice",make("sheet.rhythm.Voice",chords.get(i),measure));
        boolean unknown=name.equals("empty-read-retains-clock");
        if(!unknown) {
            call(chords.get(0),"setTimeOffset",make("math.Rational",1,4));
            call(chords.get(1),"setTimeOffset",make("math.Rational",1,2));
        }
        List<Object> members=new ArrayList<>();
        if(name.equals("different-members-have-independent-times")) {
            for(int i=0;i<2;i++)members.add(make("sheet.rhythm.Slot$MeasureSlot",i+1,measure,List.of(chords.get(i))));
        } else members.add(make("sheet.rhythm.Slot$MeasureSlot",1,measure,List.of(chords.get(0),chords.get(1))));
        if(unknown)call(members.get(0),"setTimeOffset",make("math.Rational",1,8));
        Object slot=make("sheet.rhythm.Slot$CompoundSlot",1,measure,members);
        Object rhythm=make("sheet.rhythm.MeasureRhythm",measure);
        if((Boolean)field(rhythm,"implicitTuplets"))throw new IllegalStateException("nonimplicit comparison required");
        var ctor=c("sheet.rhythm.MeasureRhythm$SlotMapper").getDeclaredConstructor(c("sheet.rhythm.MeasureRhythm"),c("sheet.rhythm.Slot$CompoundSlot"));
        ctor.setAccessible(true);
        Object mapper=ctor.newInstance(rhythm,slot);
        List<Object> before=new ArrayList<>(),times=new ArrayList<>();
        for(Object n:members)before.add(obj("id",call(n,"getId"),"chord_ids",ids((Collection<?>)call(n,"getChords"),chords),"time_offset",fraction(call(n,"getTimeOffset"))));
        for(Object ch:chords)times.add(fraction(call(ch,"getTimeOffset")));
        boolean ok=(Boolean)call(mapper,"mapChords");
        List<Object> after=new ArrayList<>();
        for(Object n:members)after.add(obj("id",call(n,"getId"),"time_offset",fraction(call(n,"getTimeOffset"))));
        return obj("name",name,"input",obj("members",before,"chord_times",times),
                "output",obj("ok",ok,"members",after,"remaining_rookies",ids((Collection<?>)field(mapper,"rookies"),chords),
                        "mapping",mappingState(field(mapper,"mapping"),chords),"black",pairs(field(mapper,"blackList"),chords)));
    }
    public static void main(String[] args) throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
        Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");
        Object sys=((List<?>)call(sheet,"getSystems")).get(0);
        Object measure=call(((List<?>)call(sys,"getStacks")).get(1),"getFirstMeasure");
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),
                obj("original_source","9e1e55cd2746037d059345881c53e6a6754bffbd",
                        "scope","three nonimplicit original mapChords grouped-clock controls with already-owned chords; no active matching or automatic image polyphony",
                        "controls",List.of(inspectClock(measure,"different-members-have-independent-times"),inspectClock(measure,"conflicting-known-times-in-one-member"),inspectClock(measure,"empty-read-retains-clock"))));
        System.exit(0);
    }
}
