// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import java.util.List;
import java.awt.*;
import java.lang.reflect.Method;
import com.fasterxml.jackson.databind.ObjectMapper;
public class TupletSansProbe extends BeamRestClearProbe {
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

            List<Object> rests=new ArrayList<>(),baseline=new ArrayList<>(),rows=new ArrayList<>(),voiceRows=new ArrayList<>();
            Object t=make("sig.inter.TupletInter",null,c("glyph.Shape").getField("TUPLET_SIX").get(null),.8);call(t,"setImplicit",true);call(t,"setManual",true);call(sig,"addVertex",t);call(measure,"addInter",t);labels.put(t,"tuplet0");
            Object relation=make("sig.relation.ChordTupletRelation");call(relation,"setManual",true);call(sig,"addEdge",chords.get(0),t,relation);
            Object before=frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source));
            for(int index:List.of(0,1)){Object ch=chords.get(index),note=((List<?>)call(ch,"getMembers")).get(0),savedShape=call(note,"getShape"),savedDots=field(ch,"sig.inter.AbstractChordInter","dotsNumber");
                for(String shapeName:List.of("LONG_REST","BREVE_REST","WHOLE_REST","HALF_REST","QUARTER_REST","EIGHTH_REST","NOTEHEAD_VOID","NOTEHEAD_BLACK","G_CLEF"))for(int dots:List.of(0,1,2,9)) {
                    Object shape=c("glyph.Shape").getField(shapeName).get(null);setCached(note,"shape",shape);setCached(ch,"dotsNumber",dots);Object duration=null;String error=null;
                    try{duration=call(ch,"getDurationSansTuplet");}catch(Exception ex){Throwable cause=ex;while(cause.getCause()!=null)cause=cause.getCause();error=cause.getClass().getSimpleName();}
                    rows.add(obj("chord",labels.get(ch),"shape",shapeName,"dots",dots,"duration",rational(duration),"error",error));
                }
                setCached(note,"shape",savedShape);setCached(ch,"dotsNumber",savedDots);
            }
            for(Object voice:voices){List<?> members=(List<?>)call(voice,"getChords");List<Object> savedDots=new ArrayList<>();for(Object ch:members)savedDots.add(field(ch,"sig.inter.AbstractChordInter","dotsNumber"));
                for(int dots:List.of(0,1,2,9)){for(Object ch:members)setCached(ch,"dotsNumber",dots);voiceRows.add(obj("voice",labels.get(voice),"chords",names(members,labels),"dots",dots,"sum",rational(call(voice,"getDurationSansTuplet")),"retained_end",rational(call(voice,"getDuration"))));}
                for(int i=0;i<members.size();i++)setCached(members.get(i),"dotsNumber",savedDots.get(i));
            }
            Object after=frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source));
            cases.add(obj("name",mode,"declared_last_offset",offset.toString(),"declared_rests",List.of(),"declared_beams",List.of(),"declared_links",List.of(),"declared_tuplets",List.of(obj("label","tuplet0","shape","TUPLET_SIX","implicit",true,"manual",true,"unassigned",false,"bounds",null)),"declared_tuple_links",List.of(obj("source","chord0","target","tuplet0","manual",true)),"rows",rows,"voice_rows",voiceRows,"same_tuplet",call(sig,"containsVertex",t),"same_manual_relation",((Boolean)call(sig,"containsEdge",relation))&&((Boolean)call(relation,"isManual")),"before",before,"after",after));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","original sans-Tuplet chord/Voice durations with retained real manual SIX relation, dots1/2 and ignored9, measure-rest-shape bypass, old actual Voice/Slot clocks; declared private shape fields only","cases",cases));
    }
}
