// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import java.util.List;
import java.awt.*;
import java.lang.reflect.Method;
import com.fasterxml.jackson.databind.ObjectMapper;
public class NarrowSlotOwnersProbe extends BeamRestClearProbe {
    static void setCached(Object object,String name,Object value)throws Exception {for(Class<?> type=object.getClass();type!=null;type=type.getSuperclass())try{java.lang.reflect.Field f=type.getDeclaredField(name);f.setAccessible(true);f.set(object,value);return;}catch(NoSuchFieldException ex){}throw new NoSuchFieldException(name);}
    public static void main(String[] args)throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);List<Object> cases=new ArrayList<>();
        for(String mode:List.of("short_shared_first","short_distinct_first","excess_shared_first","excess_distinct_first")) {
            Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
            Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),system=((List<?>)call(sheet,"getSystems")).get(0),sig=call(system,"getSig");
            List<?> stacks=(List<?>)call(system,"getStacks");for(Object s:stacks)call(make("sheet.rhythm.PageRhythm",call(system,"getPage")),"reprocessStack",s);
            Object stack=stacks.get(0),measure=((List<?>)call(stack,"getMeasures")).get(0);
            List<Object> chords=new ArrayList<>((Collection<?>)call(measure,"getHeadChords"));chords.sort(Comparator.comparingInt((Object ch)->{try{return ((java.awt.Rectangle)call(ch,"getBounds")).x;}catch(Exception e){throw new RuntimeException(e);}}).thenComparingInt(ch->{try{return ((java.awt.Rectangle)call(ch,"getBounds")).y;}catch(Exception e){throw new RuntimeException(e);}}));
            Map<Object,String> labels=new IdentityHashMap<>();Map<Object,Object> slots=new IdentityHashMap<>();for(int i=0;i<chords.size();i++){Object ch=chords.get(i);labels.put(ch,"chord"+i);slots.put(ch,call(ch,"getSlot"));int j=0;for(Object n:(Collection<?>)call(ch,"getMembers"))labels.put(n,"head"+i+"_"+j++);}
            List<Object> voices=new ArrayList<>((Collection<?>)call(measure,"getVoices"));for(int i=0;i<voices.size();i++)labels.put(voices.get(i),"voice"+i);
            Object old=voices.get(0),seed=call(old,"getFirstChord"),last=call(old,"getLastChord"),offset=mode.startsWith("short")?make("math.Rational",1,8):make("math.Rational",5,4);call(last,"setTimeOffset",offset);call(old,"checkDuration");call(seed,"setManual",true);
            Object group=call(seed,"getBeamGroup"),source=((List<?>)call(group,"getBeams")).get(0);labels.put(group,"source_group");labels.put(source,"source_beam");
            List<Object> oldRelations=new ArrayList<>();for(Object r:(Collection<?>)call(sig,"edgeSet")){labels.put(r,"relation"+oldRelations.size());oldRelations.add(r);}

            List<Object> rests=new ArrayList<>(),baseline=new ArrayList<>(),steps=new ArrayList<>();
            Object n0=make("sheet.rhythm.Slot$MeasureSlot",7,measure,List.of(chords.get(0))),n1=make("sheet.rhythm.Slot$MeasureSlot",7,measure,List.of(chords.get(1))),n2=make("sheet.rhythm.Slot$MeasureSlot",8,measure,List.of(chords.get(0)));List<Object> narrow=List.of(n0,n1,n2);for(int i=0;i<narrow.size();i++)labels.put(narrow.get(i),"narrow"+i);
            List<Object> second=mode.endsWith("distinct_first")?List.of(n2,n0):List.of(n0,n2);Object c0=make("sheet.rhythm.Slot$CompoundSlot",3,measure,List.of(n0,n1)),c1=make("sheet.rhythm.Slot$CompoundSlot",3,measure,second);List<Object> compounds=List.of(c0,c1);labels.put(c0,"compound0");labels.put(c1,"compound1");
            for(String phase:List.of("created","shared_exact","distinct_quarter","second_exact","shared_null")){
                if(phase.equals("shared_exact"))call(n0,"setTimeOffset",make("math.Rational",7,64));
                if(phase.equals("distinct_quarter"))call(n1,"setTimeOffset",make("math.Rational",1,4));
                if(phase.equals("second_exact"))call(n2,"setTimeOffset",make("math.Rational",3,16));
                if(phase.equals("shared_null"))call(n0,"setTimeOffset",(Object)null);
                List<Object> ns=new ArrayList<>(),cs=new ArrayList<>(),bindings=new ArrayList<>();for(Object n:narrow)ns.add(obj("label",labels.get(n),"id",call(n,"getId"),"chords",names((Collection<?>)call(n,"getChords"),labels),"time",rational(call(n,"getTimeOffset"))));
                for(Object compound:compounds){List<Object> lookup=new ArrayList<>();for(Object ch:List.of(chords.get(0),chords.get(1)))lookup.add(obj("chord",labels.get(ch),"narrow",labels.get(call(compound,"getNarrowSlot",ch))));cs.add(obj("label",labels.get(compound),"id",call(compound,"getId"),"members",names((Collection<?>)call(compound,"getMembers"),labels),"chords",names((Collection<?>)call(compound,"getChords"),labels),"lookup",lookup));}
                for(Object ch:List.of(chords.get(0),chords.get(1)))bindings.add(obj("chord",labels.get(ch),"slot",labels.get(call(ch,"getSlot"))));
                steps.add(obj("phase",phase,"narrow",ns,"compounds",cs,"bindings",bindings,"state",frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source))));
            }
            cases.add(obj("name",mode,"declared_last_offset",offset.toString(),"declared_rests",List.of(),"declared_narrow",List.of(obj("label","narrow0","id",7,"chords",List.of("chord0")),obj("label","narrow1","id",7,"chords",List.of("chord1")),obj("label","narrow2","id",8,"chords",List.of("chord0"))),"declared_compounds",List.of(obj("label","compound0","id",3,"members",List.of("narrow0","narrow1")),obj("label","compound1","id",3,"members",names(second,labels))),"phases",steps));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","actual source narrow-slot constructors: shared original member vs distinct owners with reused IDs; first-containing member/duplicate Chord list/current CompoundSlot backrefs and nullable exact clocks, old Voice/Stack slots retained","cases",cases));
    }
}
