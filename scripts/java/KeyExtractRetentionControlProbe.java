// SPDX-License-Identifier: AGPL-3.0-or-later
// Explicit injected-state controls on genuine original glyph/evaluation/alter.
// Recognition still uses original KeyExtractor.extractAlter; no native output.
import org.audiveris.omr.sheet.Book;
import java.util.*;
import java.awt.Rectangle;

public class KeyExtractRetentionControlProbe extends KeyBuilderProcessProbe {
    public KeyExtractRetentionControlProbe(Book b,SortedSet<Integer> ids){super(b,ids);}
    static void restore(Object item,Object eval,Object glyph,Object alter,Object rect,Object grade,Object context)throws Exception{
        call(item,"setRect",rect);put(item,"eval",eval);put(item,"glyph",glyph);put(item,"alter",alter);put(alter,"grade",grade);put(alter,"ctxGrade",context);
    }
    static String outcome(String name,String before,Object result,Object item,Object expectedEval,Object expectedGlyph,Object expectedAlter)throws Exception{
        return "{\"name\":"+q(name)+",\"controlled_input\":true,\"before\":"+before+",\"after\":"+slice(item)
            +",\"return_is_null\":"+(result==null)+",\"return_is_existing_alter\":"+(result==expectedAlter)
            +",\"eval_identity_retained\":"+(call(item,"getEval")==expectedEval)+",\"glyph_identity_retained\":"+(call(item,"getGlyph")==expectedGlyph)
            +",\"alter_identity_retained\":"+(call(item,"getAlter")==expectedAlter)+"}";
    }
    @Override protected String additionalControls() throws Exception{
        Class<?> staffClass=Class.forName("org.audiveris.omr.sheet.Staff"),roiClass=Class.forName("org.audiveris.omr.sheet.key.KeyRoi"),shapeClass=Class.forName("org.audiveris.omr.glyph.Shape");
        Object extractor=Class.forName("org.audiveris.omr.sheet.key.KeyExtractor").getConstructor(staffClass).newInstance(activeStaff);
        double minGrade=(Double)Class.forName("org.audiveris.omr.glyph.Grades").getField("keyAlterMinGrade2").get(null);
        List<String> controls=new ArrayList<>();
        for(var entry:((Map<?,?>)get(activeBuilder,"shapeBuilders")).entrySet()){
            Object sb=entry.getValue(),roi=get(sb,"roi");List<?> items=(List<?>)roi;
            if(items.size()!=1)continue;
            Object item=items.get(0),oldEval=call(item,"getEval"),oldGlyph=call(item,"getGlyph"),oldAlter=call(item,"getAlter");
            if(oldEval==null||oldGlyph==null||oldAlter==null)continue;
            Object originalRect=call(item,"getRect"),grade=call(oldAlter,"getGrade"),context=call(oldAlter,"getContextualGrade");
            List<?> peaks=(List<?>)get(sb,"peaks");Set<?> shapes=Collections.singleton(entry.getKey());
            Object clef=((List<?>)get(activeBuilder,"clefs")).get(0);
            int height=(Integer)get(get(activeBuilder,"params"),"stdGlyphHeight");
            call(item,"setPitchRect",clef,entry.getKey(),height);Object croppedRect=call(item,"getRect");
            Object trial=Class.forName("org.audiveris.omr.sheet.key.KeySlice").getConstructor(Rectangle.class,roiClass).newInstance(croppedRect,roi);
            @SuppressWarnings("unchecked") List<Object> mutable=(List<Object>)roi;
            mutable.set(0,trial);
            Object trialResult;String trialState;
            try {trialResult=call(extractor,"extractAlter",roi,peaks,trial,shapes,minGrade,true);trialState=slice(trial);}
            finally{mutable.set(0,item);}
            if(trialResult==null)throw new IllegalStateException("Expected genuine lower-grade candidate in narrower crop");
            double lowerGrade=(Double)get(call(trial,"getEval"),"grade"),oldRaw=(Double)get(oldEval,"grade");
            if(!(oldRaw>lowerGrade))throw new IllegalStateException("Expected old raw eval strictly above narrower-crop candidate");
            call(trialResult,"remove");
            String before=slice(item);Object result=call(extractor,"extractAlter",roi,peaks,item,shapes,minGrade,true);
            controls.add("{\"name\":\"higher_eval_different_crop_trial\",\"controlled_input\":true,\"independent_candidate_trial\":"+trialState+",\"existing_raw_grade\":"+oldRaw+",\"new_candidate_raw_grade\":"+lowerGrade+",\"retention\":"+outcome("higher_eval_different_crop",before,result,item,oldEval,oldGlyph,oldAlter)+"}");
            restore(item,oldEval,oldGlyph,oldAlter,originalRect,grade,context);
            Object equalTrial=Class.forName("org.audiveris.omr.sheet.key.KeySlice").getConstructor(Rectangle.class,roiClass).newInstance(originalRect,roi);
            mutable.set(0,equalTrial);Object equalTrialResult;String equalTrialState;
            try {equalTrialResult=call(extractor,"extractAlter",roi,peaks,equalTrial,shapes,minGrade,true);equalTrialState=slice(equalTrial);}
            finally{mutable.set(0,item);}
            if(equalTrialResult==null || !get(call(equalTrial,"getEval"),"grade").equals(get(oldEval,"grade")) || call(equalTrial,"getGlyph")!=oldGlyph)
                throw new IllegalStateException("Expected independently observed equal raw evaluation and original glyph identity");
            call(equalTrialResult,"remove");
            call(oldAlter,"setGrade",0.123456789);call(oldAlter,"setContextualGrade",0.456789123);
            before=slice(item);result=call(extractor,"extractAlter",roi,peaks,item,shapes,minGrade,true);
            String equalityOutcome=outcome("equal_eval_same_glyph_retains_injected_grade_and_cache",before,result,item,oldEval,oldGlyph,oldAlter);
            controls.add(equalityOutcome.substring(0,equalityOutcome.length()-1)+",\"independent_candidate_trial\":"+equalTrialState+",\"raw_grade_equal\":true,\"candidate_glyph_is_original\":true}");
            restore(item,oldEval,oldGlyph,oldAlter,originalRect,grade,context);
            Object lowEval=Class.forName("org.audiveris.omr.classifier.Evaluation").getConstructor(shapeClass,double.class).newInstance(entry.getKey(),0.001);
            put(item,"eval",lowEval);call(item,"setRect",new Rectangle(0,0,22,40));
            before=slice(item);result=call(extractor,"extractAlter",roi,peaks,item,shapes,minGrade,true);
            controls.add(outcome("low_grade_null_return_preserves_old_alter",before,result,item,lowEval,oldGlyph,oldAlter));
            restore(item,oldEval,oldGlyph,oldAlter,originalRect,grade,context);
        }
        if(controls.isEmpty())throw new IllegalStateException("Control requires a genuine one-item recognized key");
        return "["+String.join(",",controls)+"]";
    }
}
