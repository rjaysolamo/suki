#!/usr/bin/env python3
"""Create synthetic local catalog fixtures through the API. Never run in production."""
import http.cookiejar, json, os, pathlib, secrets, urllib.request, urllib.error

base=os.environ.get('API_URL','http://127.0.0.1:8080')
if not base.startswith(('http://127.0.0.1:', 'http://localhost:')):
    raise SystemExit('Local seeding requires a loopback API URL')
jar=http.cookiejar.CookieJar()
client=urllib.request.build_opener(urllib.request.HTTPCookieProcessor(jar))
def call(path,method='GET',body=None):
    data=json.dumps(body).encode() if body is not None else None
    request=urllib.request.Request(base+'/api'+path,data,{'Content-Type':'application/json','X-Suki-Request':'1'},method=method)
    with client.open(request) as response:return json.load(response)
def login(email,password):
    jar.clear();call('/auth/login','POST',{'email':email,'password':password})

admin_email=os.environ['BOOTSTRAP_ADMIN_EMAIL'];admin_password=os.environ['BOOTSTRAP_ADMIN_PASSWORD']
credentials_path=pathlib.Path(os.environ.get('SEED_CREDENTIALS_FILE',str(pathlib.Path.home()/'runtime/suki-api/seed-accounts.json')))
credentials_path.parent.mkdir(parents=True,exist_ok=True)
credentials_path.parent.chmod(0o700)
accounts=json.loads(credentials_path.read_text()) if credentials_path.exists() else {}
vendors=['Aling Nena’s Fresh Picks','Bong’s Vegetable Stall','Davao Fruit Corner','Jun’s Fresh Catch','Lito’s Meat Shop']
products=[('Fresh Tomatoes','Kamatis','Vegetables',8000,'tomatoes',0),('Benguet Carrots','Karot','Vegetables',10000,'carrots',1),('Fresh Eggplant','Talong','Vegetables',7500,'eggplant',0),('Sweet Lacatan Bananas','Saging','Fruits',9000,'bananas',2),('Davao Sweet Mangoes','Mangga','Fruits',16000,'mangoes',2),('Fresh Green Cabbage','Repolyo','Vegetables',7000,'cabbage',1),('Fresh Market Fish','Isda','Fish & Seafood',22000,'fish',3),('Fresh Whole Chicken','Manok','Meat & Poultry',20000,'chicken',4)]
for i,shop in enumerate(vendors):
    key=f'vendor{i+1}'
    if key not in accounts:
        accounts[key]={'email':f'{key}@suki.invalid','password':secrets.token_urlsafe(24)}
        credentials_path.write_text(json.dumps(accounts));credentials_path.chmod(0o600)
    account=accounts[key]
    try:result=call('/auth/register','POST',{'name':shop,'email':account['email'],'phone':'09000000000','password':account['password'],'role':'vendor','shop_name':shop});account['id']=result['id']
    except urllib.error.HTTPError as error:
        if error.code!=409:raise
    login(admin_email,admin_password)
    if 'id' not in account:
        account['id']=next(u['id'] for u in call('/admin/users') if u['email']==account['email'])
    call('/admin/users/'+account['id'],'PATCH',{'role':'vendor','status':'active'})
    login(account['email'],account['password'])
    existing={p['name'] for p in call('/products') if p['vendor_id']==account['id']}
    for name,local,category,price,image,owner in products:
        if owner==i and name not in existing:
            call('/products','POST',{'name':name,'local_name':local,'description':'Synthetic local catalog fixture. Replace with vendor-maintained product information before launch.','category':category,'unit':'kg','price_centavos':price,'image':'/images/'+image+'.jpg','available':True})
credentials_path.write_text(json.dumps(accounts));credentials_path.chmod(0o600)
print('Seeded synthetic vendor profiles and catalog. Credentials saved privately; GCash and Maps remain unconfigured.')
