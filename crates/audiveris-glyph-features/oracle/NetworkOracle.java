// SPDX-License-Identifier: AGPL-3.0-or-later
// Oracle adapter, 2026-10-02. Calls unchanged Audiveris 5.11.0 algorithms.
import java.nio.file.*;
import java.io.*;
import java.util.*;
import java.util.zip.*;
import javax.xml.parsers.*;
import org.w3c.dom.*;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.audiveris.omr.math.NeuralNetwork;
import org.audiveris.omr.math.PoorManAlgebra.Nd4j;
import org.audiveris.omr.math.PoorManAlgebra.INDArray;

public class NetworkOracle {
    static Map<String,Object> obj(Object... args) {
        Map<String,Object> m = new LinkedHashMap<>();
        for (int i=0;i<args.length;i+=2) m.put((String)args[i],args[i+1]);
        return m;
    }
    static double[] vector(ZipFile z, String name) throws Exception {
        DocumentBuilderFactory f=DocumentBuilderFactory.newInstance();
        f.setFeature("http://apache.org/xml/features/disallow-doctype-decl",true);
        try (InputStream s=z.getInputStream(z.getEntry(name))) {
            NodeList n=f.newDocumentBuilder().parse(s).getElementsByTagName("value");
            double[] a=new double[n.getLength()];
            for(int i=0;i<a.length;i++) a[i]=Double.parseDouble(n.item(i).getTextContent());
            return a;
        }
    }
    static Map<String,Object> evaluate(String id,double[] raw,NeuralNetwork nn,double[] means,double[] stds) {
        INDArray f=Nd4j.create(raw.clone());
        // The exact PoorManAlgebra calls used by BasicClassifier.normalize.
        f.subiRowVector(Nd4j.create(means.clone()));
        f.diviRowVector(Nd4j.create(stds.clone()));
        double[] normalized=new double[110];
        for(int i=0;i<110;i++) normalized[i]=f.getDouble(i);
        double[] hidden=new double[149], out=new double[149];
        nn.run(normalized,hidden,out);
        Integer[] order=new Integer[149];for(int i=0;i<149;i++) order[i]=i;
        Arrays.sort(order,(a,b)->{int c=Double.compare(out[b],out[a]);return c!=0?c:Integer.compare(a,b);});
        List<Object> top=new ArrayList<>();
        for(int i=0;i<5;i++)top.add(obj("index",order[i],"label",nn.getOutputLabels()[order[i]],"score",out[order[i]]));
        return obj("id",id,"raw_features",raw,"normalized",normalized,"hidden",hidden,"scores",out,"top5",top);
    }
    public static void main(String[] args) throws Exception {
        NeuralNetwork nn;double[] means,stds;
        try(ZipFile z=new ZipFile(args[0])) {
            try(InputStream in=z.getInputStream(z.getEntry("model.xml"))) { nn=NeuralNetwork.unmarshal(in); }
            means=vector(z,"means.xml");stds=vector(z,"stds.xml");
        }
        if(nn.getInputSize()!=110||nn.getHiddenSize()!=149||nn.getOutputSize()!=149)throw new AssertionError("model dimensions");
        List<Object> vectors=new ArrayList<>();
        String[] modes={"zero","one","minus_one","alternating","ramp","impulse_first","impulse_middle","impulse_last","stress_positive","stress_negative"};
        for(String mode:modes) {
            double[] target=new double[110],raw=new double[110];
            for(int i=0;i<110;i++) {
                switch(mode) {
                    case "one":target[i]=1;break;
                    case "minus_one":target[i]=-1;break;
                    case "alternating":target[i]=(i%2==0?1:-1);break;
                    case "ramp":target[i]=(i-55)/20.0;break;
                    case "impulse_first":target[i]=(i==0?1:0);break;
                    case "impulse_middle":target[i]=(i==55?1:0);break;
                    case "impulse_last":target[i]=(i==109?1:0);break;
                    case "stress_positive":target[i]=20;break;
                    case "stress_negative":target[i]=-20;break;
                }
                raw[i]=means[i]+stds[i]*target[i];
            }
            Map<String,Object> row=evaluate("features_"+mode,raw,nn,means,stds);
            row.put("requested_normalized",target);vectors.add(row);
        }
        for(int index=0;index<110;index++) {
            double[] raw=means.clone();raw[index]+=stds[index];
            vectors.add(evaluate("basis_"+index,raw,nn,means,stds));
        }
        Random random=new Random(5110149L);
        for(int k=0;k<16;k++) {
            double[] raw=new double[110];
            for(int i=0;i<110;i++)raw[i]=means[i]+stds[i]*(random.nextDouble()*8-4);
            vectors.add(evaluate("seeded_"+k,raw,nn,means,stds));
        }
        Map<String,Object> root=obj("schema","audiveris-classifier-golden-v1","audiveris_tag","5.11.0","audiveris_commit","9e1e55cd2746037d059345881c53e6a6754bffbd","model_sha256","327194c47116a8ebf5771d04cea08a4e563ec93d2955e1efd990a54a9245f29e","input_labels",nn.getInputLabels(),"output_labels",nn.getOutputLabels(),"means",means,"stds",stds,"feature_cases",vectors);
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(new File(args[1]),root);
        System.err.println("Generated "+vectors.size()+" feature cases using "+NeuralNetwork.class.getProtectionDomain().getCodeSource().getLocation());
    }
}
