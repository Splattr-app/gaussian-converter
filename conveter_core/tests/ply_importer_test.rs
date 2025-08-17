#[cfg(test)]
use conveter_core::Importer;
use conveter_core::formats::ply::PlyImporter;
use std::io::Cursor; // TIn-memory reader

#[test]
fn ply_importer_success() {
  let sample_ply_data = r#"ply
format ascii 1.0
element vertex 2
property float x
property float y
property float z        
property float nx
property float ny
property float nz
property float f_dc_0
property float f_dc_1
property float f_dc_2
property float f_rest_0         
property float f_rest_1
property float f_rest_2
property float f_rest_3
property float f_rest_4
property float f_rest_5
property float f_rest_6
property float f_rest_7
property float f_rest_8
property float f_rest_9
property float f_rest_10
property float f_rest_11
property float f_rest_12
property float f_rest_13
property float f_rest_14
property float f_rest_15
property float f_rest_16
property float f_rest_17
property float f_rest_18
property float f_rest_19
property float f_rest_20
property float f_rest_21
property float f_rest_22
property float f_rest_23
property float f_rest_24
property float f_rest_25
property float f_rest_26
property float f_rest_27
property float f_rest_28
property float f_rest_29
property float f_rest_30
property float f_rest_31
property float f_rest_32
property float f_rest_33
property float f_rest_34
property float f_rest_35
property float f_rest_36
property float f_rest_37
property float f_rest_38
property float f_rest_39
property float f_rest_40
property float f_rest_41
property float f_rest_42
property float f_rest_43
property float f_rest_44
property float opacity
property float scale_0
property float scale_1
property float scale_2
property float rot_0
property float rot_1
property float rot_2
property float rot_3
end_header
-0.37904015 0.0016934633 0.0067018056 0 0 0 -2.0724692 -2.0561755 -2.1148913 0.01796955 0.037105832 0.26626846 -0.13593107 -0.0019182944 0.117212705 0.066411406 -0.07922326 0.1866683 -0.14814459 -0.12625426 -0.092399135 -0.01042401 0.03961014 0.011356966 0.01641241 0.028952168 0.26359588 -0.13751353 -0.0035359813 0.0924597 0.050215878 -0.07316286 0.16539887 -0.14700699 -0.10993294 -0.106098086 -0.027348816 0.049000986 0.010465037 0.030529652 0.035236355 0.2371268 -0.13656521 -0.00047623453 0.086775266 0.05186912 -0.07666206 0.17974722 -0.12158594 -0.09541294 -0.09566374 -0.016787965 0.023918556 0.00094769296 14.655018 -2.8385267 -4.059616 -9.483267 -1.0241892 -0.53246504 0.68165195 0.182208
-0.27332094 0.023124091 -0.33375475 0 0 0 -0.36709127 -0.43788102 -0.42018273 0.0057332683 0.05278704 0.24297205 -0.19215158 0.000015837346 0.017771488 -0.11089609 -0.06589045 0.081130214 0.008250793 0.007624493 -0.094845735 -0.15574777 0.020495798 -0.11981493 0.039251037 0.031223925 0.19802268 -0.20118569 -0.033441886 0.03213683 -0.10079756 -0.058582522 0.067221776 -0.0045770854 -0.03554812 -0.121091284 -0.14446107 0.008793312 -0.120233506 0.02750991 0.029116591 0.20882143 -0.22425015 -0.004233506 -0.01218978 -0.12682378 -0.043143056 0.08931449 0.048926592 0.020401904 -0.086050324 -0.114629254 0.018081024 -0.14749396 11.957442 -15.78881 -3.580685 -2.9721582 -0.9023066 0.43795106 0.25956437 -0.14243974
"#;

  // Wrap the string data in a Cursor to make it behave like a file.
  let mut reader = Cursor::new(sample_ply_data.as_bytes());

  let result = PlyImporter::import(&mut reader);

  assert!(
    result.is_ok(),
    "Importer should successfully parse valid PLY data"
  );

  let scene = result.unwrap();
  assert_eq!(scene.splats.len(), 2, "Should parse exactly 2 splats");
}

#[test]
fn ply_importer_failure_on_bad_data() {
  // 1. Create invalid data.
  let bad_data = "this is not a ply file";

  // 2. Create the reader.
  let mut reader = Cursor::new(bad_data.as_bytes());

  // 3. Call the function.
  let result = PlyImporter::import(&mut reader);

  // 4. Assert that the function correctly returned an error.
  assert!(result.is_err(), "Importer should fail on invalid data");
}
