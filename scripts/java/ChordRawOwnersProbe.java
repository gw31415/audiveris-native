// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import java.util.List;
import java.awt.*;
import java.lang.reflect.Method;
import com.fasterxml.jackson.databind.ObjectMapper;
public class ChordRawOwnersProbe extends BeamRestClearProbe {
    static void setCached(Object object,String name,Object value)throws Exception {for(Class<?> type=object.getClass();type!=null;type=type.getSuperclass())try{java.lang.reflect.Field f=type.getDeclaredField(name);f.setAccessible(true);f.set(object,value);return;}catch(NoSuchFieldException ex){}throw new NoSuchFieldException(name);}
    public static void main(String[] args)throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);List<Object> cases=new ArrayList<>();
        for(String mode:List.of("short","excess")) {
            Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
            Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),system=((List<?>)call(sheet,"getSystems")).get(0),sig=call(system,"getSig");
            List<?> stacks=(List<?>)call(system,"getStacks");for(Object s:stacks)call(make("sheet.rhythm.PageRhythm",call(system,"getPage")),"reprocessStack",s);
            Object stack=stacks.get(0),measure=((List<?>)call(stack,"getMeasures")).get(0);
            List<Object> chords=new ArrayList<>((Collection<?>)call(measure,"getHeadChords"));chords.sort(Comparator.comparingInt((Object ch)->{try{return ((java.awt.Rectangle)call(ch,"getBounds")).x;}catch(Exception e){throw new RuntimeException(e);}}).thenComparingInt(ch->{try{return ((java.awt.Rectangle)call(ch,"getBounds")).y;}catch(Exception e){throw new RuntimeException(e);}}));
            Map<Object,String> labels=new IdentityHashMap<>();Map<Object,Object> slots=new IdentityHashMap<>();for(int i=0;i<chords.size();i++){Object ch=chords.get(i);labels.put(ch,"chord"+i);slots.put(ch,call(ch,"getSlot"));int j=0;for(Object n:(Collection<?>)call(ch,"getMembers"))labels.put(n,"head"+i+"_"+j++);}
            List<Object> voices=new ArrayList<>((Collection<?>)call(measure,"getVoices"));for(int i=0;i<voices.size();i++)labels.put(voices.get(i),"voice"+i);
            Object old=voices.get(0),seed=call(old,"getFirstChord"),last=call(old,"getLastChord"),offset=mode.equals("short")?make("math.Rational",1,8):make("math.Rational",5,4);call(last,"setTimeOffset",offset);call(old,"checkDuration");call(seed,"setManual",true);
            Object group=call(seed,"getBeamGroup"),source=((List<?>)call(group,"getBeams")).get(0);labels.put(group,"source_group");labels.put(source,"source_beam");
            List<Object> oldRelations=new ArrayList<>();for(Object r:(Collection<?>)call(sig,"edgeSet")){labels.put(r,"relation"+oldRelations.size());oldRelations.add(r);}

            List<Object> rests=new ArrayList<>(),baseline=new ArrayList<>(),rows=new ArrayList<>();Object before=frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source));
            Object ch=chords.get(1),note=((List<?>)call(ch,"getMembers")).get(0),opposite=((List<?>)call(chords.get(0),"getMembers")).get(0),savedShape=call(note,"getShape"),savedDots=field(ch,"sig.inter.AbstractChordInter","dotsNumber"),oldVoice=call(ch,"getVoice"),oldSlot=call(ch,"getSlot"),stem=call(ch,"getStem"),mirror=null;List<Object> flags=new ArrayList<>(),edges=new ArrayList<>();
            setCached(ch,"dotsNumber",9);
            for(String phase:List.of("void","mirror_void","mirror_small_void","mirror_flag2","mirror_flag2_flag1")){
                setCached(note,"shape",c("glyph.Shape").getField(phase.equals("mirror_small_void")?"NOTEHEAD_VOID_SMALL":"NOTEHEAD_VOID").get(null));
                if(phase.equals("mirror_void")){mirror=make("sig.relation.MirrorRelation");call(mirror,"setManual",true);call(sig,"addEdge",note,opposite,mirror);}
                if(phase.equals("mirror_flag2")||phase.equals("mirror_flag2_flag1")){int value=phase.equals("mirror_flag2")?2:1;Object flag=make("sig.inter.FlagInter",null,c("glyph.Shape").getField("FLAG_"+value).get(null),.8);call(flag,"setBounds",new Rectangle(200,150,15,35));call(flag,"setStaff",call(system,"getFirstStaff"));call(flag,"setManual",true);call(sig,"addVertex",flag);Object edge=make("sig.relation.FlagStemRelation");call(edge,"setManual",true);call(sig,"addEdge",flag,stem,edge);flags.add(flag);edges.add(edge);}
                rows.add(obj("phase",phase,"shape",call(note,"getShape").toString(),"declared_dots",9,"declared_flag_values",flags.stream().map(f->{try{return call(f,"getValue");}catch(Exception e){throw new RuntimeException(e);}}).toList(),"mirror_registered",mirror!=null&&((Boolean)call(sig,"containsEdge",mirror)),"mirror_manual",mirror==null?null:call(mirror,"isManual"),"raw",rational(call(ch,"getDurationSansDotOrTuplet")),"beams_or_flags",call(ch,"getBeamsOrFlagsNumber"),"same_chord",((Collection<?>)call(measure,"getHeadChords")).contains(ch),"same_voice",call(ch,"getVoice")==oldVoice,"same_slot",call(ch,"getSlot")==oldSlot,"flag_relation_manual",edges.stream().map(e->{try{return call(e,"isManual");}catch(Exception ex){throw new RuntimeException(ex);}}).toList()));
            }
            for(Object edge:edges)call(sig,"removeEdge",edge);for(Object flag:flags)call(flag,"remove");if(mirror!=null)call(sig,"removeEdge",mirror);setCached(note,"shape",savedShape);setCached(ch,"dotsNumber",savedDots);
            Object after=frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source));cases.add(obj("name",mode,"declared_last_offset",offset.toString(),"declared_rests",List.of(),"declared_beams",List.of(),"declared_links",List.of(),"declared_tuplets",List.of(),"declared_tuple_links",List.of(),"rows",rows,"before",before,"after",after));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","original raw getter on real warmed beamed Chord with actual manual mirror and flag objects; void promotion only exact NOTEHEAD_VOID, summed Beam+flag2+flag1; dots9 ignored; actual owner/Voice/Slot retained; private note shape control; not complete Flag/Mirror callback admission or normal XML scope","cases",cases));
    }
}
