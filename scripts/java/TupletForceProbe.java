// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import java.util.List;
import java.awt.*;
import java.lang.reflect.Method;
import com.fasterxml.jackson.databind.ObjectMapper;
public class TupletForceProbe extends BeamRestClearProbe {
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

            List<Object> rests=new ArrayList<>(),baseline=new ArrayList<>(),geometry=new ArrayList<>();
            for(int i=0;i<chords.size();i++){Object ch=chords.get(i);int x=301+i*31,y=201+i*3,w=11,h=23;Rectangle bounds=new Rectangle(x,y,w,h);Point head=new Point(x+w/2,y+9),tail=new Point(head.x,head.y-40);setCached(ch,"bounds",bounds);setCached(ch,"headLocation",head);setCached(ch,"tailLocation",tail);geometry.add(obj("chord",labels.get(ch),"bounds",List.of(x,y,w,h),"head",List.of(head.x,head.y),"tail",List.of(tail.x,tail.y)));}
            Object stub=call(sheet,"getStub");call(call(stub,"getMusicFamilyParam"),"setSpecific",c("ui.symbol.MusicFamily").getField("Bravura").get(null));
            Object generator=make("sheet.rhythm.TupletGenerator",measure),rhythm=make("sheet.rhythm.MeasureRhythm",measure);Method force=c("sheet.rhythm.TupletGenerator").getDeclaredMethod("forceTuplets",c("sheet.rhythm.Voice"));force.setAccessible(true);Method remove=c("sheet.rhythm.MeasureRhythm").getDeclaredMethod("removeTuplets",Collection.class);remove.setAccessible(true);
            Object time=call(stack,"getCurrentTimeSignature"),beat=time==null?make("math.Rational",1,4):call(time,"getBeatValue");List<Object> scenarios=new ArrayList<>();
            for(Object voice:voices){List<Object> created=new ArrayList<>(),passes=new ArrayList<>();
                for(int pass=0;pass<2;pass++) {
                    List<Object> callbacks=new ArrayList<>();Object listener=java.lang.reflect.Proxy.newProxyInstance(TupletForceProbe.class.getClassLoader(),new Class<?>[]{Class.forName("org.jgrapht.event.GraphListener")},(proxy,method,eventArgs)->{
                        if(method.getName().equals("equals"))return proxy==eventArgs[0];if(method.getName().equals("hashCode"))return System.identityHashCode(proxy);if(method.getName().equals("toString"))return "actual force Tuplet observer";
                        Object t=null;String phase=null;
                        if(method.getName().equals("vertexAdded")){t=call(eventArgs[0],"getVertex");phase="vertex_registered";if(c("sig.inter.TupletInter").isInstance(t)){labels.put(t,"generated"+created.size());created.add(t);}}
                        if(method.getName().equals("edgeAdded")&&c("sig.relation.ChordTupletRelation").isInstance(call(eventArgs[0],"getEdge"))){t=call(eventArgs[0],"getEdgeTarget");phase="chord_tuplet_added";}
                        if(t!=null&&c("sig.inter.TupletInter").isInstance(t))callbacks.add(obj("phase",phase,"tuplet",labels.get(t),"sig_attached",call(t,"getSig")!=null,"abnormal",call(t,"isAbnormal"),"measure_contains",((Collection<?>)call(measure,"getTuplets")).contains(t),"stack_contains",((Collection<?>)call(stack,"getTuplets")).contains(t),"clocks",observe(sig,measure,stack,chords,voices,labels,slots,oldRelations)));
                        return null;});call(sig,"addGraphListener",listener);
                    Object before=frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source));Collection<?> result=(Collection<?>)force.invoke(generator,voice);call(sig,"removeGraphListener",listener);List<Object> tuples=new ArrayList<>();
                    for(Object t:created){Rectangle b=(Rectangle)call(t,"getBounds");tuples.add(obj("label",labels.get(t),"shape",call(t,"getShape").toString(),"bounds",List.of(b.x,b.y,b.width,b.height),"chords",names((Collection<?>)call(t,"getChords"),labels),"manual",call(t,"isManual"),"implicit",call(t,"isImplicit"),"abnormal",call(t,"isAbnormal")));}
                    passes.add(obj("pass",pass,"returned",names(result,labels),"tuplets",tuples,"callbacks",callbacks,"before",before,"after",frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source))));
                }
                remove.invoke(rhythm,created);scenarios.add(obj("voice",labels.get(voice),"passes",passes,"removed",frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source))));
            }
            cases.add(obj("name",mode,"declared_last_offset",offset.toString(),"interline",call(call(sheet,"getScale"),"getInterline"),"family",call(stub,"getMusicFamily").toString(),"actual_beat",rational(beat),"declared_rests",List.of(),"declared_beams",List.of(),"declared_links",List.of(),"declared_tuplets",List.of(),"declared_tuple_links",List.of(),"geometry",geometry,"scenarios",scenarios));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","original private forceTuplets on actual Voice/current original time signature; explicit control-local Bravura/cached Chord geometry, fresh allocation/second call reuses same Tuplets, callbacks/old clocks held; automatic font/signature binding/eligibility/second rhythm pass not claimed","cases",cases));
    }
}
