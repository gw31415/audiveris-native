// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import java.util.List;
import java.awt.*;
import java.lang.reflect.Method;
import com.fasterxml.jackson.databind.ObjectMapper;
public class TupletRawBaseProbe extends BeamRestClearProbe {
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

            List<Object> rests=new ArrayList<>(),baseline=new ArrayList<>(),rows=new ArrayList<>();
            Object before=frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source));
            List<Object> shapes=new ArrayList<>();for(Object shape:c("glyph.Shape").getEnumConstants())if(call(shape,"getNoteDuration")!=null)shapes.add(shape);shapes.add(c("glyph.Shape").getField("G_CLEF").get(null));
            for(int index:List.of(0,1)){Object ch=chords.get(index),note=((List<?>)call(ch,"getMembers")).get(0),savedShape=call(note,"getShape"),savedDots=field(ch,"sig.inter.AbstractChordInter","dotsNumber");
                for(Object shape:shapes)for(int dots:List.of(0,1,2,9)){setCached(note,"shape",shape);setCached(ch,"dotsNumber",dots);String error=null;Object raw=null;try{raw=call(ch,"getDurationSansDotOrTuplet");}catch(Exception ex){Throwable cause=ex;while(cause.getCause()!=null)cause=cause.getCause();error=cause.getClass().getSimpleName();}rows.add(obj("chord",labels.get(ch),"shape",shape.toString(),"dots",dots,"shape_is_rest",call(shape,"isRest"),"beams_or_flags",call(ch,"getBeamsOrFlagsNumber"),"raw",rational(raw),"error",error));}
                setCached(note,"shape",savedShape);setCached(ch,"dotsNumber",savedDots);
            }
            List<Object> declaredTuplets=new ArrayList<>(),tuplets=new ArrayList<>(),baseRows=new ArrayList<>();
            for(int index:List.of(0,1)){Object ch=chords.get(index),t=make("sig.inter.TupletInter",null,c("glyph.Shape").getField(index==0?"TUPLET_THREE":"TUPLET_SIX").get(null),.8);call(t,"setImplicit",true);call(t,"setManual",index==1);call(sig,"addVertex",t);call(measure,"addInter",t);labels.put(t,"tuplet"+index);tuplets.add(t);declaredTuplets.add(obj("label","tuplet"+index,"shape",index==0?"TUPLET_THREE":"TUPLET_SIX","implicit",true,"manual",index==1,"unassigned",false,"bounds",null));Object e=make("sig.relation.ChordTupletRelation");call(e,"setManual",index==1);call(sig,"addEdge",ch,t,e);Object savedDots=field(ch,"sig.inter.AbstractChordInter","dotsNumber");setCached(ch,"dotsNumber",9);Object first=call(t,"getBaseDuration");setCached(ch,"dotsNumber",0);Object retry=call(t,"getBaseDuration");baseRows.add(obj("tuplet","tuplet"+index,"source",labels.get(ch),"declared_first_dots",9,"first_base",rational(first),"same_base",first==retry));setCached(ch,"dotsNumber",savedDots);}
            Object after=frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source));
            cases.add(obj("name",mode,"declared_last_offset",offset.toString(),"declared_rests",List.of(),"declared_beams",List.of(),"declared_links",List.of(),"declared_tuplets",declaredTuplets,"declared_tuple_links",List.of(),"raw_rows",rows,"base_rows",baseRows,"before",before,"after",after));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","original uncached raw sans-dot-or-tuplet first actual note duration, all Shape.getNoteDuration aliases plus G_CLEF null and first Tuplet base under ignored dots9; private field declarations only, no setShape workflow or image/XML scope widening; old owners/clocks preserved","cases",cases));
    }
}
