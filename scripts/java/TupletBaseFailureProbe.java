// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import java.util.List;
import java.awt.*;
import java.lang.reflect.Method;
import com.fasterxml.jackson.databind.ObjectMapper;
public class TupletBaseFailureProbe extends BeamRestClearProbe {
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

            List<Object> rests=new ArrayList<>(),baseline=new ArrayList<>(),scenarios=new ArrayList<>();Object before=frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source));
            for(int index=0;index<3;index++){int[] incoming=index==0?new int[]{0,1}:index==1?new int[]{1,0}:new int[]{0,1};int bad=index==0?1:0;Object note=((List<?>)call(chords.get(bad),"getMembers")).get(0),savedShape=call(note,"getShape");setCached(note,"shape",c("glyph.Shape").getField("G_CLEF").get(null));Object t=make("sig.inter.TupletInter",null,c("glyph.Shape").getField("TUPLET_THREE").get(null),.8);call(t,"setImplicit",true);call(t,"setManual",true);call(sig,"addVertex",t);call(measure,"addInter",t);List<Object> links=new ArrayList<>();for(int sourceIndex:incoming){Object edge=make("sig.relation.ChordTupletRelation");call(edge,"setManual",links.isEmpty());call(sig,"addEdge",chords.get(sourceIndex),t,edge);links.add(edge);}
                String error=null;Object result=null;try{result=call(t,"getBaseDuration");}catch(Exception ex){Throwable cause=ex;while(cause.getCause()!=null)cause=cause.getCause();error=cause.getClass().getSimpleName();}Object cached=field(t,"sig.inter.TupletInter","baseDuration"),retry=call(t,"getBaseDuration"),partBefore=call(t,"getPart"),staff=call(t,"getStaff"),partAfter=call(t,"getPart");call(t,"setStaff",(Object)null);Object partCleared=call(t,"getPart");
                scenarios.add(obj("name",index==0?"known_then_unknown_beamed":index==1?"known_then_null_lone":"null_lone_then_known","incoming",Arrays.stream(incoming).mapToObj(i->"chord"+i).toList(),"bad_source","chord"+bad,"declared_shape","G_CLEF","error",error,"result",rational(result),"cached_after_first",rational(cached),"retry",rational(retry),"same_cached_retry",cached==retry,"part_before_staff",partBefore==null?null:call(partBefore,"getId"),"staff",staff==null?null:call(staff,"getId"),"part_after_staff",partAfter==null?null:call(partAfter,"getId"),"part_after_clear_staff",partCleared==null?null:call(partCleared,"getId"),"manual",call(t,"isManual"),"relation_manual",links.stream().map(e->{try{return call(e,"isManual");}catch(Exception ex){throw new RuntimeException(ex);}}).toList()));
                setCached(note,"shape",savedShape);call(measure,"removeInter",t);call(stack,"removeInter",t);call(t,"remove");
            }
            Object after=frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source));cases.add(obj("name",mode,"declared_last_offset",offset.toString(),"declared_rests",List.of(),"declared_beams",List.of(),"declared_links",List.of(),"declared_tuplets",List.of(obj("label","tuplet0","shape","TUPLET_THREE","implicit",true,"manual",true,"unassigned",false,"bounds",null)),"declared_tuple_links",List.of(),"scenarios",scenarios,"before",before,"after",after));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","original lazy Tuplet base partial cache on later raw failure/null, cached retry without re-reading source; inherited getPart reads existing Staff field without triggering lazy getStaff, retains Part across Staff clear; actual manual relation/Chord/Voice/Slot owners; private shape control only; not original outer transaction rollback","cases",cases));
    }
}
