// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

public class PreviousSlotMergeProbe extends CandidateReferenceProbe {
    static Object inspectMerge(Object measure, String name) throws Exception {
        call(measure,"resetRhythm");
        List<Object> chords=new ArrayList<>((Collection<?>)call(measure,"getStandardChords"));
        Object sig=call(call(call(measure,"getStack"),"getSystem"),"getSig");
        Object edge=null;
        if(!name.equals("known-not-equal")) {
            edge=make("sig.relation.SameTimeRelation");
            call(sig,"addEdge",chords.get(0),chords.get(1),edge);
        }
        Object rhythm=make("sheet.rhythm.MeasureRhythm",measure);
        Object retriever=make("sheet.rhythm.SlotsRetriever",measure,false);
        put(rhythm,"narrowSlotsRetriever",retriever);
        call(retriever,"buildSlots");
        List<Object> objects=new ArrayList<>();
        for(int i=0;i<chords.size();i++)objects.add(compound(measure,i+1,
                i==0 && name.equals("equal-position-removes-first-equal-object")?List.of(chords.get(0),chords.get(2)):List.of(chords.get(i))));
        List<Object> constructors=new ArrayList<>();
        IdentityHashMap<Object,Integer> keys=new IdentityHashMap<>();
        for(int i=0;i<objects.size();i++)keys.put(objects.get(i),i);
        for(Object sl:objects)constructors.add(obj("id",call(sl,"getId"),"chord_ids",ids((Collection<?>)call(sl,"getChords"),chords),
                "narrow_chord_ids",ids((Collection<?>)call(((List<?>)call(sl,"getMembers")).get(0),"getChords"),chords)));
        List ordered=(List)field(rhythm,"slots");
        ordered.addAll(objects);
        Object v1=make("sheet.rhythm.Voice",chords.get(0),measure);
        call(measure,"addVoice",v1);
        boolean known=!name.equals("unknown-equal");
        if(known)call(chords.get(0),"setTimeOffset",make("math.Rational",1,4));
        Object input=obj("relation",String.valueOf(call(retriever,"getRel",chords.get(0),chords.get(1))),
                "previous_time",fraction(call(chords.get(0),"getTimeOffset")),"current_key",1,"rookie",1,"constructors",constructors);
        var ctor=c("sheet.rhythm.MeasureRhythm$SlotMapper").getDeclaredConstructor(
                c("sheet.rhythm.MeasureRhythm"),c("sheet.rhythm.Slot$CompoundSlot"));
        ctor.setAccessible(true);
        Object mapper=ctor.newInstance(rhythm,objects.get(1));
        boolean merged;
        if(name.equals("known-equal")) {
            invoke(mapper,"createNewVoices");
            merged=!ordered.contains(objects.get(1));
        } else {
            Object v2=make("sheet.rhythm.Voice",chords.get(1),measure);
            call(measure,"addVoice",v2);
            merged=(Boolean)invoke(mapper,"mergeWithPreviousSlot",chords.get(1));
        }
        List<Object> view=new ArrayList<>(),refs=new ArrayList<>(),voices=new ArrayList<>();
        for(Object sl:ordered)view.add(obj("object_key",keys.get(sl),"id",call(sl,"getId"),
                "chord_ids",ids((Collection<?>)call(sl,"getChords"),chords)));
        for(int i=0;i<chords.size();i++) {
            Object sl=call(chords.get(i),"getSlot");
            refs.add(obj("chord",i,"object_key",keys.get(sl),"id",call(sl,"getId"),"x_offset",call(sl,"getXOffset")));
        }
        for(Object v:(List<?>)call(measure,"getVoices"))voices.add(obj("id",call(v,"getId"),"chord_ids",ids((Collection<?>)call(v,"getChords"),chords)));
        Object output=obj("merged",merged,"ordered",view,"references",refs,"voices",voices,
                "rookie_time",fraction(call(chords.get(1),"getTimeOffset")),"rookie_voice",call(call(chords.get(1),"getVoice"),"getId"),
                "previous_narrow_chord_ids",ids((Collection<?>)call(((List<?>)call(objects.get(0),"getMembers")).get(0),"getChords"),chords));
        if(edge!=null)call(sig,"removeEdge",edge);
        return obj("name",name,"comparison",name.equals("known-equal")?"createNewVoices calls mergeWithPreviousSlot":"mergeWithPreviousSlot only; newly owned Voice declared",
                "input",input,"output",output);
    }
    public static void main(String[] args) throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
        Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");
        Object sys=((List<?>)call(sheet,"getSystems")).get(0);
        Object measure=call(((List<?>)call(sys,"getStacks")).get(1),"getFirstMeasure");
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),
                obj("original_source","9e1e55cd2746037d059345881c53e6a6754bffbd",
                        "scope","four declared original previous-slot merge controls on saved chord/SIG objects; not automatic image polyphony",
                        "controls",List.of(inspectMerge(measure,"known-equal"),inspectMerge(measure,"unknown-equal"),inspectMerge(measure,"known-not-equal"),inspectMerge(measure,"equal-position-removes-first-equal-object"))));
        System.exit(0);
    }
}
