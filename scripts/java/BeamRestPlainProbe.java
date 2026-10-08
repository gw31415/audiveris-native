// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

/** Actual plain candidate ownership and insertion order before lookup geometry. */
public class BeamRestPlainProbe extends BeamRestClearProbe {
    static Object linked(Object group,List<?> seeds,boolean transitive,Map<Object,String> labels,String... kinds)throws Exception {
        Class<?>[] types=new Class<?>[kinds.length];for(int i=0;i<kinds.length;i++)types[i]=c("sig.relation."+kinds[i]);
        java.lang.reflect.Method method=c("sig.inter.BeamGroupInter").getDeclaredMethod("getLinkedRests",List.class,boolean.class,Class[].class);method.setAccessible(true);
        return names((Collection<?>)method.invoke(group,seeds,transitive,(Object)types),labels);
    }
    public static void main(String[] args)throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        List<Object> cases=new ArrayList<>();
        for(String mode:List.of("short","excess")) {
            Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
            Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),system=((List<?>)call(sheet,"getSystems")).get(0),sig=call(system,"getSig");
            List<?> stacks=(List<?>)call(system,"getStacks");for(Object s:stacks)call(make("sheet.rhythm.PageRhythm",call(system,"getPage")),"reprocessStack",s);
            Object stack=stacks.get(0),measure=((List<?>)call(stack,"getMeasures")).get(0),targetMeasure=((List<?>)call(stacks.get(1),"getMeasures")).get(0);
            List<Object> chords=new ArrayList<>((Collection<?>)call(measure,"getHeadChords"));chords.sort(Comparator.comparingInt((Object ch)->{try{return ((java.awt.Rectangle)call(ch,"getBounds")).x;}catch(Exception e){throw new RuntimeException(e);}}).thenComparingInt(ch->{try{return ((java.awt.Rectangle)call(ch,"getBounds")).y;}catch(Exception e){throw new RuntimeException(e);}}));
            Map<Object,String> labels=new IdentityHashMap<>();Map<Object,Object> slots=new IdentityHashMap<>();
            for(int i=0;i<chords.size();i++){Object ch=chords.get(i);labels.put(ch,"chord"+i);slots.put(ch,call(ch,"getSlot"));int j=0;for(Object n:(Collection<?>)call(ch,"getMembers"))labels.put(n,"head"+i+"_"+j++);}
            List<Object> voices=new ArrayList<>((Collection<?>)call(measure,"getVoices"));for(int i=0;i<voices.size();i++)labels.put(voices.get(i),"voice"+i);
            Object old=voices.get(0),seed=call(old,"getFirstChord"),outside=call(old,"getLastChord"),offset=mode.equals("short")?make("math.Rational",1,8):make("math.Rational",5,4);
            call(outside,"setTimeOffset",offset);call(old,"checkDuration");call(seed,"setManual",true);
            Object group=call(seed,"getBeamGroup"),beam=((List<?>)call(group,"getBeams")).get(0);labels.put(group,"source_group");labels.put(beam,"source_beam");
            List<Object> relations=new ArrayList<>();for(Object r:(Collection<?>)call(sig,"edgeSet")){labels.put(r,"relation"+relations.size());relations.add(r);}
            List<Object> rests=new ArrayList<>(),baseline=new ArrayList<>(),declared=new ArrayList<>();Map<String,Object> endpoints=new LinkedHashMap<>();endpoints.put("seed",seed);endpoints.put("outside",outside);
            int[] xs={355,370,390,410,420,900,630,435,910};
            for(int i=0;i<xs.length;i++) {
                Object m=(i==5||i==8)?targetMeasure:measure,rest=make("sig.inter.RestInter",glyph(xs[i],240),c("glyph.Shape").getField("QUARTER_REST").get(null),.8,call(system,"getFirstStaff"),0.),chord=make("sig.inter.RestChordInter",.8);
                call(chord,"setStaff",call(system,"getFirstStaff"));call(sig,"addVertex",rest);call(sig,"addVertex",chord);call(chord,"addMember",rest);call(m,"addInter",chord);
                labels.put(rest,"rest"+i);labels.put(chord,"rest_chord"+i);endpoints.put("rest_chord"+i,chord);rests.add(chord);
                if(i!=5&&i!=8){chords.add(chord);slots.put(chord,null);}declared.add(obj("bounds",List.of(xs[i],240,8,20),"manual",false,"source_measure",i!=5&&i!=8,"beam_link",false));
            }
            String[][] specs={{"seed","rest_chord0","NextInVoiceRelation"},{"rest_chord1","seed","NextInVoiceRelation"},{"rest_chord0","rest_chord2","NextInVoiceRelation"},{"rest_chord3","rest_chord1","NextInVoiceRelation"},{"rest_chord2","rest_chord4","NextInVoiceRelation"},{"rest_chord4","rest_chord0","NextInVoiceRelation"},{"rest_chord3","rest_chord5","NextInVoiceRelation"},{"seed","outside","NextInVoiceRelation"},{"outside","rest_chord6","NextInVoiceRelation"},{"seed","rest_chord6","SameVoiceRelation"},{"rest_chord2","rest_chord6","SameVoiceRelation"},{"seed","rest_chord2","SeparateVoiceRelation"},{"rest_chord2","rest_chord4","SeparateVoiceRelation"}};
            List<Object> links=new ArrayList<>();for(int i=0;i<specs.length;i++){String[] spec=specs[i];Object r=make("sig.relation."+spec[2]);boolean manual=i%2==0;call(r,"setManual",manual);call(sig,"addEdge",endpoints.get(spec[0]),endpoints.get(spec[1]),r);labels.put(r,"link"+i);relations.add(r);links.add(obj("source",spec[0],"target",spec[1],"kind",spec[2],"manual",manual));}
            List<Object> states=new ArrayList<>();
            for(String phase:List.of("linked","manual_beam_rest","manual_foreign","cleared","measure_reset")) {
                if(phase.equals("manual_beam_rest")){Object r=make("sig.relation.BeamRestRelation");call(r,"setManual",true);call(sig,"addEdge",beam,((List<?>)call(rests.get(0),"getMembers")).get(0),r);labels.put(r,"manual_beam_rest");relations.add(r);}
                if(phase.equals("manual_foreign")){Object r=make("sig.relation.BeamRestRelation");call(r,"setManual",true);call(sig,"addEdge",beam,((List<?>)call(rests.get(5),"getMembers")).get(0),r);labels.put(r,"foreign_manual_beam_rest");relations.add(r);}
                if(phase.equals("cleared")){java.lang.reflect.Method method=c("sheet.rhythm.MeasureRhythm").getDeclaredMethod("clearInterleavedRests");method.setAccessible(true);method.invoke(make("sheet.rhythm.MeasureRhythm",measure));}
                if(phase.equals("measure_reset"))call(measure,"resetRhythm");
                List<?> seeds=(List<?>)call(group,"getChords");List<Object> black=(List<Object>)linked(group,seeds,false,labels,"SeparateVoiceRelation"),white=(List<Object>)linked(group,seeds,true,labels,"NextInVoiceRelation");white.removeAll(black);
                Map<String,Object> queries=obj("seeds",names(seeds,labels),"next_direct",linked(group,seeds,false,labels,"NextInVoiceRelation"),"next_transitive",linked(group,seeds,true,labels,"NextInVoiceRelation"),"same_transitive",linked(group,seeds,true,labels,"SameVoiceRelation"),"mixed_transitive",linked(group,seeds,true,labels,"NextInVoiceRelation","SameVoiceRelation"),"separate_direct",black,"separate_transitive",linked(group,seeds,true,labels,"SeparateVoiceRelation"),"white",white,
                    "seed_next_relations",names((Collection<?>)call(sig,"getRelations",seed,c("sig.relation.NextInVoiceRelation")),labels));
                List<Object> measures=new ArrayList<>((Collection<?>)call(group,"getMeasures"));LinkedHashSet<Object> plain=new LinkedHashSet<>();for(Object m:measures)plain.addAll((Collection<?>)call(m,"getRestChords"));
                // linked() returns labels: retain actual RestChord refs when subtracting.
                plain.removeIf(rest->black.contains(labels.get(rest))||white.contains(labels.get(rest)));
                queries.put("measure_numbers",measures.stream().map(m->m==measure?1:2).toList());queries.put("plain",names(plain,labels));
                states.add(obj("phase",phase,"queries",queries,"state",frame(sig,measure,stack,chords,voices,labels,slots,relations,rests,baseline,List.of(beam))));
            }
            cases.add(obj("name",mode,"declared_last_offset",offset.toString(),"declared_rests",declared,"declared_links",links,"seed",labels.get(seed),"outside",labels.get(outside),"states",states));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","actual plain candidates: original getMeasures stable stack-left order and real Measure rest insertion order minus direct Separate and transitive Next-only white; manual foreign-measure BeamRest expands group ownership; all old clock/cache refs held through clear and genuine reset; polygon lookup pending","cases",cases));
    }
}
