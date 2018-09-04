pub trait UrlConvertable {
    fn to_url(&self) -> String;
}

impl UrlConvertable for String {
    fn to_url(&self) -> String {
        return self.chars()
            .map(|x| match x {
                'ı' => 'i',
                'ğ' => 'g',
                'ü' => 'u',
                'ş' => 's',
                'ö' => 'o',
                'ç' => 'c',
                '.' => '-',
                '+' => '-',

                _   => x
            })
            .filter(|x| match *x {
                '\'' => false,
                _    => true
            })
            .collect();
    }
}
