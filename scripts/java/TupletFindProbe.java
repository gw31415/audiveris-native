// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import java.util.List;
import java.awt.*;
import java.lang.reflect.Method;
import com.fasterxml.jackson.databind.ObjectMapper;
public class TupletFindProbe extends BeamRestClearProbe {
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
            for(int i=0;i<chords.size();i++){Object ch=chords.get(i);int x=301+i*31,y=201+i*3;Rectangle bounds=new Rectangle(x,y,11,23);Point head=new Point(x+5,y+9),tail=new Point(head.x,head.y-40);setCached(ch,"bounds",bounds);setCached(ch,"headLocation",head);setCached(ch,"tailLocation",tail);geometry.add(obj("chord",labels.get(ch),"bounds",List.of(x,y,11,23),"head",List.of(head.x,head.y),"tail",List.of(tail.x,tail.y)));}
            Object stub=call(sheet,"getStub");call(call(stub,"getMusicFamilyParam"),"setSpecific",c("ui.symbol.MusicFamily").getField("Bravura").get(null));
            Object generator=make("sheet.rhythm.TupletGenerator",measure),rhythm=make("sheet.rhythm.MeasureRhythm",measure);Method remove=c("sheet.rhythm.MeasureRhythm").getDeclaredMethod("removeTuplets",Collection.class);remove.setAccessible(true);List<Object> scenarios=new ArrayList<>();
            Object savedExpected=call(stack,"getExpectedDuration");List<Object> savedFirst=new ArrayList<>(),dotChords=new ArrayList<>(),savedDots=new ArrayList<>();for(Object voice:voices)savedFirst.add(call(call(voice,"getFirstChord"),"getTimeOffset"));for(Object ch:(List<?>)call(voices.get(0),"getChords")){dotChords.add(ch);savedDots.add(field(ch,"sig.inter.AbstractChordInter","dotsNumber"));}
            for(String pattern:List.of("original","null_first_all","nonzero_first_all","dot_voice0"))for(String expectedText:Arrays.asList("1","2/3",null,"0")) {
                Object expected=null;if(expectedText!=null){String[] pieces=expectedText.split("/");expected=make("math.Rational",Integer.parseInt(pieces[0]),pieces.length==2?Integer.parseInt(pieces[1]):1);}call(stack,"setExpectedDuration",(Object)expected);
                List<Object> firsts=new ArrayList<>();for(int i=0;i<voices.size();i++){Object first=call(voices.get(i),"getFirstChord"),clock=pattern.equals("null_first_all")?null:pattern.equals("nonzero_first_all")?make("math.Rational",1,8):savedFirst.get(i);call(first,"setTimeOffset",(Object)clock);firsts.add(obj("voice",labels.get(voices.get(i)),"chord",labels.get(first),"time",rational(clock)));}
                if(pattern.equals("dot_voice0"))for(Object ch:dotChords)setCached(ch,"dotsNumber",1);
                Object before=frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source));List<Object> created=new ArrayList<>();
                Object listener=java.lang.reflect.Proxy.newProxyInstance(TupletFindProbe.class.getClassLoader(),new Class<?>[]{Class.forName("org.jgrapht.event.GraphListener")},(proxy,method,eventArgs)->{if(method.getName().equals("equals"))return proxy==eventArgs[0];if(method.getName().equals("hashCode"))return System.identityHashCode(proxy);if(method.getName().equals("toString"))return "actual eligibility allocation observer";if(method.getName().equals("vertexAdded")){Object t=call(eventArgs[0],"getVertex");if(c("sig.inter.TupletInter").isInstance(t))created.add(t);}return null;});call(sig,"addGraphListener",listener);
                Collection<?> result=null;String error=null;try{result=(Collection<?>)call(generator,"findImplicitTuplets");}catch(Exception ex){Throwable cause=ex;while(cause.getCause()!=null)cause=cause.getCause();error=cause.getClass().getSimpleName();}call(sig,"removeGraphListener",listener);List<Object> selected=new ArrayList<>();Set<Object> seen=Collections.newSetFromMap(new IdentityHashMap<>());if(result!=null)for(Object t:result){Object v=call(t,"getVoice");if(seen.add(v))selected.add(labels.get(v));}
                Object after=frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source));remove.invoke(rhythm,created);
                scenarios.add(obj("pattern",pattern,"expected",expectedText,"firsts",firsts,"dots",pattern.equals("dot_voice0")?1:0,"dot_chords",names(dotChords,labels),"selected",selected,"error",error,"returned_count",result==null?null:result.size(),"fresh_count",created.size(),"before",before,"generated",after,"removed",frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source))));
                for(int i=0;i<voices.size();i++)call(call(voices.get(i),"getFirstChord"),"setTimeOffset",savedFirst.get(i));for(int i=0;i<dotChords.size();i++)setCached(dotChords.get(i),"dotsNumber",savedDots.get(i));call(stack,"setExpectedDuration",savedExpected);
            }
            cases.add(obj("name",mode,"declared_last_offset",offset.toString(),"family",call(stub,"getMusicFamily").toString(),"interline",call(call(sheet,"getScale"),"getInterline"),"declared_rests",List.of(),"declared_beams",List.of(),"declared_links",List.of(),"declared_tuplets",List.of(),"declared_tuple_links",List.of(),"geometry",geometry,"scenarios",scenarios));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","actual original findImplicitTuplets on existing ordinary Voices; ZERO/null/nonzero first clock, exact dotted sans-Tuplet ratio, missing/zero expected, fresh count and returned actual Voice identities; explicit cached geometry/Bravura controls, no rest-Voice or automatic native font/signature binding claim","cases",cases));
    }
}
